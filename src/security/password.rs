use argon2::{
    password_hash::{PasswordHash, PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use rand::rngs::OsRng;
use crate::config::auth::AuthConfig;

/// Production-ready Argon2id password hasher
///
/// This hasher implements the Argon2id algorithm, which is the recommended
/// password hashing algorithm as of 2024. It provides strong resistance against
/// both side-channel attacks and GPU-based brute force attacks.
///
/// # Security Features
/// - Argon2id algorithm (hybrid of Argon2i and Argon2d)
/// - Configurable memory cost, time cost, and parallelism
/// - Unique salt per password (generated via cryptographically secure RNG)
/// - Version 0x13 (latest stable version)
///
/// # Performance Characteristics
/// With default settings (64MB memory, 3 iterations, 4 threads):
/// - Hashing time: 100-500ms (intentionally slow to prevent brute force)
/// - Verification time: ~100-500ms (same as hashing)
///
/// # Example
/// ```no_run
/// use lighter_auth::config::auth::AuthConfig;
/// use lighter_auth::security::PasswordHasher;
///
/// let config = AuthConfig::default();
/// let hasher = PasswordHasher::from_config(&config).unwrap();
///
/// // Hash a password
/// let hash = hasher.hash("my_secure_password").unwrap();
///
/// // Verify a password
/// let is_valid = hasher.verify("my_secure_password", &hash).unwrap();
/// assert!(is_valid);
/// ```
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    /// Create PasswordHasher from AuthConfig
    ///
    /// # Arguments
    /// * `config` - AuthConfig containing Argon2 parameters
    ///
    /// # Returns
    /// * `Ok(PasswordHasher)` - Successfully created hasher
    /// * `Err(argon2::password_hash::Error)` - Invalid parameters
    ///
    /// # Errors
    /// Returns error if Argon2 parameters are invalid (e.g., memory cost too high)
    ///
    /// # Example
    /// ```no_run
    /// use lighter_auth::config::auth::AuthConfig;
    /// use lighter_auth::security::PasswordHasher;
    ///
    /// let config = AuthConfig::default();
    /// let hasher = PasswordHasher::from_config(&config).unwrap();
    /// ```
    #[tracing::instrument(skip(config))]
    pub fn from_config(config: &AuthConfig) -> Result<Self, argon2::password_hash::Error> {
        let params = Params::new(
            config.argon2.memory_cost,      // 65536 (64 MB)
            config.argon2.time_cost,        // 3 iterations
            config.argon2.parallelism,      // 4 threads
            Some(config.argon2.hash_length as usize),// 32 bytes
        )?;

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        Ok(Self { argon2 })
    }

    /// Hash a password using Argon2id
    ///
    /// Generates a unique salt and produces a hash in PHC string format:
    /// `$argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>`
    ///
    /// # Arguments
    /// * `password` - Plain text password to hash
    ///
    /// # Returns
    /// * `Ok(String)` - PHC-formatted hash string
    /// * `Err(argon2::password_hash::Error)` - Hashing failed
    ///
    /// # Security
    /// - Each call generates a unique salt (cryptographically secure)
    /// - Salt is stored in the hash string (no separate storage needed)
    /// - Hash output is safe to store in database
    ///
    /// # Performance
    /// With default settings: ~100-500ms per call (intentionally slow)
    ///
    /// # Example
    /// ```no_run
    /// # use lighter_auth::config::auth::AuthConfig;
    /// # use lighter_auth::security::PasswordHasher;
    /// # let config = AuthConfig::default();
    /// # let hasher = PasswordHasher::from_config(&config).unwrap();
    /// let hash = hasher.hash("my_password").unwrap();
    /// assert!(hash.starts_with("$argon2id$"));
    /// ```
    #[tracing::instrument(skip(self, password))]
    pub fn hash(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self.argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// # Arguments
    /// * `password` - Plain text password to verify
    /// * `hash` - PHC-formatted hash string (from `hash()` method)
    ///
    /// # Returns
    /// * `Ok(true)` - Password matches hash
    /// * `Ok(false)` - Password does not match hash
    /// * `Err(argon2::password_hash::Error)` - Invalid hash format or verification failed
    ///
    /// # Security
    /// - Constant-time comparison to prevent timing attacks
    /// - Extracts salt and parameters from hash string automatically
    ///
    /// # Example
    /// ```no_run
    /// # use lighter_auth::config::auth::AuthConfig;
    /// # use lighter_auth::security::PasswordHasher;
    /// # let config = AuthConfig::default();
    /// # let hasher = PasswordHasher::from_config(&config).unwrap();
    /// # let hash = hasher.hash("correct_password").unwrap();
    /// let is_valid = hasher.verify("correct_password", &hash).unwrap();
    /// assert!(is_valid);
    ///
    /// let is_invalid = hasher.verify("wrong_password", &hash).unwrap();
    /// assert!(!is_invalid);
    /// ```
    #[tracing::instrument(skip(self, password, hash))]
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Check if password hash needs rehashing (e.g., params changed)
    ///
    /// Use this to upgrade password hashes when you change Argon2 parameters
    /// in your configuration. Call this during login and rehash if needed.
    ///
    /// # Arguments
    /// * `hash` - PHC-formatted hash string to check
    ///
    /// # Returns
    /// * `Ok(true)` - Hash needs rehashing (outdated parameters)
    /// * `Ok(false)` - Hash is up-to-date
    /// * `Err(argon2::password_hash::Error)` - Invalid hash format
    ///
    /// # Use Case
    /// ```no_run
    /// # use lighter_auth::config::auth::AuthConfig;
    /// # use lighter_auth::security::PasswordHasher;
    /// # let config = AuthConfig::default();
    /// # let hasher = PasswordHasher::from_config(&config).unwrap();
    /// # let old_hash = String::from("$argon2id$v=19$m=65536,t=3,p=4$...$...");
    /// # let password = "user_password";
    /// // During login:
    /// if hasher.verify(password, &old_hash).unwrap() {
    ///     if hasher.needs_rehash(&old_hash).unwrap() {
    ///         let new_hash = hasher.hash(password).unwrap();
    ///         // Update database with new_hash
    ///     }
    /// }
    /// ```
    #[tracing::instrument(skip(self, hash))]
    pub fn needs_rehash(&self, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;

        // Check if algorithm matches (argon2id identifier is "argon2id")
        if parsed_hash.algorithm.as_str() != "argon2id" {
            return Ok(true);
        }

        // Extract params from the hash (returns Option, so use unwrap_or to trigger rehash if missing)
        let m_cost = parsed_hash.params.get_decimal("m").unwrap_or(0);
        let t_cost = parsed_hash.params.get_decimal("t").unwrap_or(0);
        let p_cost = parsed_hash.params.get_decimal("p").unwrap_or(0);

        // Get current params
        let current = self.argon2.params();

        // Check if params match current config
        if m_cost != current.m_cost() ||
           t_cost != current.t_cost() ||
           p_cost != current.p_cost()
        {
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::auth::{AuthConfig, Argon2Config};

    fn test_config() -> AuthConfig {
        AuthConfig {
            token_expiration: 3600,
            token_cleanup_interval: 900,
            max_sessions: 5,
            session_cache_ttl: 300,
            password_hash_algorithm: crate::config::auth::PasswordHashAlgorithm::Argon2,
            argon2: Argon2Config {
                memory_cost: 65536,
                time_cost: 3,
                parallelism: 4,
                hash_length: 32,
                salt_length: 16,
            },
            jwt: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_hash_produces_different_hashes() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash1 = hasher.hash("password123").unwrap();
        let hash2 = hasher.hash("password123").unwrap();

        assert_ne!(hash1, hash2, "Same password should produce different hashes due to different salts");
    }

    #[tokio::test]
    async fn test_verify_correct_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("correct_password").unwrap();

        assert!(hasher.verify("correct_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_verify_wrong_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("correct_password").unwrap();

        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_hash_format() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("test_password").unwrap();

        // Argon2 hashes start with $argon2id$
        assert!(hash.starts_with("$argon2id$"));
    }

    #[tokio::test]
    async fn test_needs_rehash_same_params() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("password").unwrap();

        assert!(!hasher.needs_rehash(&hash).unwrap());
    }

    #[tokio::test]
    async fn test_needs_rehash_different_params() {
        let config1 = test_config();
        let hasher1 = PasswordHasher::from_config(&config1).unwrap();
        let hash = hasher1.hash("password").unwrap();

        // Create hasher with different params
        let mut config2 = test_config();
        config2.argon2.time_cost = 5;
        let hasher2 = PasswordHasher::from_config(&config2).unwrap();

        assert!(hasher2.needs_rehash(&hash).unwrap());
    }

    #[tokio::test]
    async fn test_empty_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("").unwrap();

        assert!(hasher.verify("", &hash).unwrap());
        assert!(!hasher.verify("not_empty", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_long_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let long_password = "a".repeat(1000);
        let hash = hasher.hash(&long_password).unwrap();

        assert!(hasher.verify(&long_password, &hash).unwrap());
    }

    #[tokio::test]
    async fn test_unicode_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let unicode_password = "пароль123🔐";
        let hash = hasher.hash(unicode_password).unwrap();

        assert!(hasher.verify(unicode_password, &hash).unwrap());
    }

    #[tokio::test]
    async fn test_hashing_performance() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();

        let start = std::time::Instant::now();
        let _hash = hasher.hash("benchmark_password").unwrap();
        let duration = start.elapsed();

        // Argon2 with these params should take 50-1000ms in release mode
        // Debug mode can be slower (up to 10s is acceptable for testing)
        assert!(duration.as_millis() >= 50, "Hashing too fast: {}ms (vulnerable to brute force)", duration.as_millis());

        #[cfg(debug_assertions)]
        {
            assert!(duration.as_millis() <= 10000, "Hashing too slow even for debug: {}ms", duration.as_millis());
        }

        #[cfg(not(debug_assertions))]
        {
            assert!(duration.as_millis() <= 1000, "Hashing too slow: {}ms (bad UX)", duration.as_millis());
        }

        println!("Hashing took {}ms", duration.as_millis());
    }

    #[tokio::test]
    async fn test_verify_invalid_hash_format() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();

        let result = hasher.verify("password", "invalid_hash");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_from_config_creates_hasher() {
        let config = test_config();
        let result = PasswordHasher::from_config(&config);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hash_contains_params() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("test").unwrap();

        // Should contain memory cost
        assert!(hash.contains("m=65536"));
        // Should contain time cost
        assert!(hash.contains("t=3"));
        // Should contain parallelism
        assert!(hash.contains("p=4"));
        // Should contain version
        assert!(hash.contains("v=19"));
    }

    #[tokio::test]
    async fn test_multiple_hashes_all_unique() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let password = "same_password";

        let hashes: Vec<String> = (0..5)
            .map(|_| hasher.hash(password).unwrap())
            .collect();

        // All hashes should be unique due to different salts
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j]);
            }
        }

        // But all should verify correctly
        for hash in hashes {
            assert!(hasher.verify(password, &hash).unwrap());
        }
    }

    #[tokio::test]
    async fn test_case_sensitive_verification() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("Password123").unwrap();

        assert!(hasher.verify("Password123", &hash).unwrap());
        assert!(!hasher.verify("password123", &hash).unwrap());
        assert!(!hasher.verify("PASSWORD123", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_whitespace_matters() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let hash = hasher.hash("password").unwrap();

        assert!(hasher.verify("password", &hash).unwrap());
        assert!(!hasher.verify(" password", &hash).unwrap());
        assert!(!hasher.verify("password ", &hash).unwrap());
        assert!(!hasher.verify(" password ", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_special_characters_in_password() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();
        let special_passwords = vec![
            "p@ssw0rd!",
            "test#123$456",
            "user^name&pass",
            "(){}[]<>",
            "quotes\"'`",
        ];

        for password in special_passwords {
            let hash = hasher.hash(password).unwrap();
            assert!(hasher.verify(password, &hash).unwrap());
        }
    }

    #[tokio::test]
    async fn test_needs_rehash_invalid_hash() {
        let hasher = PasswordHasher::from_config(&test_config()).unwrap();

        let result = hasher.needs_rehash("invalid_hash");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_hashing() {
        use std::sync::Arc;
        use std::thread;

        let hasher = Arc::new(PasswordHasher::from_config(&test_config()).unwrap());
        let mut handles = vec![];

        // Spawn 4 threads to hash concurrently
        for i in 0..4 {
            let hasher_clone = Arc::clone(&hasher);
            let handle = thread::spawn(move || {
                let password = format!("password_{}", i);
                let hash = hasher_clone.hash(&password).unwrap();
                hasher_clone.verify(&password, &hash).unwrap()
            });
            handles.push(handle);
        }

        // All threads should succeed
        for handle in handles {
            assert!(handle.join().unwrap());
        }
    }
}
