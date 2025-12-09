use serde::{Deserialize, Serialize};
use lighter_common::config::*;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Token expiration time in seconds
    #[serde(default = "default_token_expiration")]
    pub token_expiration: u64,
    /// Token cleanup interval in seconds
    #[serde(default = "default_token_cleanup_interval")]
    pub token_cleanup_interval: u64,
    /// Maximum number of sessions per user
    #[serde(default = "default_max_sessions")]
    pub max_sessions: u32,
    /// Session cache TTL in seconds
    #[serde(default = "default_session_cache_ttl")]
    pub session_cache_ttl: u64,
    /// Password hash algorithm
    #[serde(default = "default_password_hash_algorithm")]
    pub password_hash_algorithm: PasswordHashAlgorithm,
    /// Argon2 configuration
    #[serde(default = "Argon2Config::default")]
    pub argon2: Argon2Config,
    /// JWT configuration
    #[serde(default = "JwtConfig::default")]
    pub jwt: JwtConfig,
}

/// Argon2 password hashing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Config {
    /// Memory cost in KB (64MB = 65536 KB)
    #[serde(default = "default_argon2_memory_cost")]
    pub memory_cost: u32,
    /// Time cost (iterations)
    #[serde(default = "default_argon2_time_cost")]
    pub time_cost: u32,
    /// Parallelism (number of threads)
    #[serde(default = "default_argon2_parallelism")]
    pub parallelism: u32,
    /// Hash length in bytes
    #[serde(default = "default_argon2_hash_length")]
    pub hash_length: u32,
    /// Salt length in bytes
    #[serde(default = "default_argon2_salt_length")]
    pub salt_length: u32,
}

/// JWT (JSON Web Token) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Enable JWT authentication
    #[serde(default = "default_jwt_enabled")]
    pub enabled: bool,
    /// JWT secret key (should be set via environment variable)
    #[serde(default = "default_jwt_secret")]
    pub secret: String,
    /// JWT algorithm
    #[serde(default = "default_jwt_algorithm")]
    pub algorithm: JwtAlgorithm,
    /// JWT issuer
    #[serde(default = "default_jwt_issuer")]
    pub issuer: String,
    /// JWT audience
    #[serde(default = "default_jwt_audience")]
    pub audience: String,
}

/// Password hash algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PasswordHashAlgorithm {
    /// Argon2 (recommended)
    Argon2,
    /// SHA-256 (less secure, for compatibility)
    Sha256,
}

/// JWT algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwtAlgorithm {
    /// HMAC with SHA-256
    HS256,
    /// HMAC with SHA-384
    HS384,
    /// HMAC with SHA-512
    HS512,
    /// RSA with SHA-256
    RS256,
}

// Default functions for AuthConfig
fn default_token_expiration() -> u64 {
    3600 // 1 hour
}

fn default_token_cleanup_interval() -> u64 {
    900 // 15 minutes
}

fn default_max_sessions() -> u32 {
    5
}

fn default_session_cache_ttl() -> u64 {
    300 // 5 minutes
}

fn default_password_hash_algorithm() -> PasswordHashAlgorithm {
    PasswordHashAlgorithm::Argon2
}

// Default functions for Argon2Config
fn default_argon2_memory_cost() -> u32 {
    65536 // 64 MB
}

fn default_argon2_time_cost() -> u32 {
    3
}

fn default_argon2_parallelism() -> u32 {
    4
}

fn default_argon2_hash_length() -> u32 {
    32
}

fn default_argon2_salt_length() -> u32 {
    16
}

// Default functions for JwtConfig
fn default_jwt_enabled() -> bool {
    false
}

fn default_jwt_secret() -> String {
    String::new()
}

fn default_jwt_algorithm() -> JwtAlgorithm {
    JwtAlgorithm::HS256
}

fn default_jwt_issuer() -> String {
    "lighter-auth".to_string()
}

fn default_jwt_audience() -> String {
    "lighter-api".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_expiration: default_token_expiration(),
            token_cleanup_interval: default_token_cleanup_interval(),
            max_sessions: default_max_sessions(),
            session_cache_ttl: default_session_cache_ttl(),
            password_hash_algorithm: default_password_hash_algorithm(),
            argon2: Argon2Config::default(),
            jwt: JwtConfig::default(),
        }
    }
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost: default_argon2_memory_cost(),
            time_cost: default_argon2_time_cost(),
            parallelism: default_argon2_parallelism(),
            hash_length: default_argon2_hash_length(),
            salt_length: default_argon2_salt_length(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            enabled: default_jwt_enabled(),
            secret: default_jwt_secret(),
            algorithm: default_jwt_algorithm(),
            issuer: default_jwt_issuer(),
            audience: default_jwt_audience(),
        }
    }
}

impl Validate for AuthConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.token_expiration == 0 {
            return Err(ConfigError::ValidationError("auth.token_expiration must be > 0".to_string()));
        }
        if self.token_cleanup_interval == 0 {
            return Err(ConfigError::ValidationError("auth.token_cleanup_interval must be > 0".to_string()));
        }
        if self.max_sessions == 0 {
            return Err(ConfigError::ValidationError("auth.max_sessions must be > 0".to_string()));
        }
        if self.session_cache_ttl == 0 {
            return Err(ConfigError::ValidationError("auth.session_cache_ttl must be > 0".to_string()));
        }
        self.argon2.validate()?;
        self.jwt.validate()?;
        Ok(())
    }
}

impl Validate for Argon2Config {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.memory_cost == 0 {
            return Err(ConfigError::ValidationError("auth.argon2.memory_cost must be > 0".to_string()));
        }
        if self.time_cost == 0 {
            return Err(ConfigError::ValidationError("auth.argon2.time_cost must be > 0".to_string()));
        }
        if self.parallelism == 0 {
            return Err(ConfigError::ValidationError("auth.argon2.parallelism must be > 0".to_string()));
        }
        if self.hash_length == 0 {
            return Err(ConfigError::ValidationError("auth.argon2.hash_length must be > 0".to_string()));
        }
        if self.salt_length == 0 {
            return Err(ConfigError::ValidationError("auth.argon2.salt_length must be > 0".to_string()));
        }
        Ok(())
    }
}

impl Validate for JwtConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.secret.is_empty() {
            return Err(ConfigError::ValidationError("auth.jwt.secret must be set when JWT is enabled".to_string()));
        }
        if self.issuer.is_empty() {
            return Err(ConfigError::ValidationError("auth.jwt.issuer cannot be empty".to_string()));
        }
        if self.audience.is_empty() {
            return Err(ConfigError::ValidationError("auth.jwt.audience cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl WithDefaults for AuthConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for Argon2Config {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for JwtConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    async fn test_auth_config_defaults() {
        let config = AuthConfig::default();
        assert_eq!(config.token_expiration, 3600);
        assert_eq!(config.token_cleanup_interval, 900);
        assert_eq!(config.max_sessions, 5);
        assert_eq!(config.session_cache_ttl, 300);
        assert_eq!(config.password_hash_algorithm, PasswordHashAlgorithm::Argon2);
    }

    #[test]
    async fn test_argon2_config_defaults() {
        let config = Argon2Config::default();
        assert_eq!(config.memory_cost, 65536); // 64 MB
        assert_eq!(config.time_cost, 3);
        assert_eq!(config.parallelism, 4);
        assert_eq!(config.hash_length, 32);
        assert_eq!(config.salt_length, 16);
    }

    #[test]
    async fn test_jwt_config_defaults() {
        let config = JwtConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.secret, "");
        assert_eq!(config.algorithm, JwtAlgorithm::HS256);
        assert_eq!(config.issuer, "lighter-auth");
        assert_eq!(config.audience, "lighter-api");
    }

    #[test]
    async fn test_auth_config_validation_zero_token_expiration() {
        let config = AuthConfig {
            token_expiration: 0,
            ..AuthConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_jwt_config_validation_enabled_without_secret() {
        let config = JwtConfig {
            enabled: true,
            secret: "".to_string(),
            ..JwtConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_jwt_config_validation_enabled_with_secret() {
        let config = JwtConfig {
            enabled: true,
            secret: "my-secret-key".to_string(),
            ..JwtConfig::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    async fn test_argon2_config_validation_zero_memory_cost() {
        let config = Argon2Config {
            memory_cost: 0,
            ..Argon2Config::default()
        };
        assert!(config.validate().is_err());
    }
}
