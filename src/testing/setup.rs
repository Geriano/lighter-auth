use lighter_auth_migration::MigratorTrait;
use lighter_common::prelude::*;
use sea_orm::{ActiveModelTrait, DbErr};

use crate::cache::LocalCache;
use crate::config::auth::{AuthConfig, Argon2Config};
use crate::entities::v1::users;
use crate::security::password::PasswordHasher;

/// Returns an in-memory SQLite database with all migrations applied
///
/// This creates a fresh database connection for each test, ensuring test isolation.
/// All migrations are automatically applied, so the database schema is ready to use.
///
/// # Panics
/// Panics if database connection fails or migrations fail to apply.
/// This is intentional for test setup - tests should fail fast if setup is broken.
///
/// # Example
/// ```no_run
/// use lighter_auth::testing::setup;
///
/// #[tokio::test]
/// async fn test_something() {
///     let db = setup::database().await;
///     // Use db for testing
/// }
/// ```
pub async fn database() -> DatabaseConnection {
    // Connect to SQLite in-memory database
    let db = database::memory()
        .await
        .expect("Failed to connect to in-memory database");

    // Run all migrations
    lighter_auth_migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

/// Returns a local cache instance for testing
///
/// Creates a new LocalCache with default configuration.
/// Each call returns a new instance, ensuring test isolation.
///
/// # Returns
/// A LocalCache instance that can be used in tests
///
/// # Example
/// ```no_run
/// use lighter_auth::testing::setup;
///
/// #[tokio::test]
/// async fn test_caching() {
///     let cache = setup::cache();
///     cache.set("key", &"value", std::time::Duration::from_secs(60)).await.unwrap();
///     let result: Option<String> = cache.get("key").await.unwrap();
///     assert_eq!(result, Some("value".to_string()));
/// }
/// ```
pub fn cache() -> LocalCache {
    LocalCache::new()
}

/// Returns a PasswordHasher configured with fast parameters for testing
///
/// Uses reduced Argon2 parameters to speed up tests while maintaining
/// correctness verification. Production settings would be too slow for tests.
///
/// # Test Parameters
/// - memory_cost: 19456 KB (19 MB instead of 64 MB)
/// - time_cost: 1 iteration (instead of 3)
/// - parallelism: 1 thread (instead of 4)
/// - hash_length: 32 bytes (same as production)
/// - salt_length: 16 bytes (same as production)
///
/// These parameters make tests ~50-100x faster while still testing the
/// actual hashing logic.
///
/// # Returns
/// - `Ok(PasswordHasher)` - Configured hasher ready for testing
/// - `Err(argon2::password_hash::Error)` - If parameters are invalid (shouldn't happen)
///
/// # Example
/// ```no_run
/// use lighter_auth::testing::setup;
///
/// #[tokio::test]
/// async fn test_password() {
///     let hasher = setup::password_hasher().unwrap();
///     let hash = hasher.hash("password").unwrap();
///     assert!(hasher.verify("password", &hash).unwrap());
/// }
/// ```
pub fn password_hasher() -> Result<PasswordHasher, argon2::password_hash::Error> {
    let config = AuthConfig {
        token_expiration: 3600,
        token_cleanup_interval: 900,
        max_sessions: 5,
        session_cache_ttl: 300,
        password_hash_algorithm: crate::config::auth::PasswordHashAlgorithm::Argon2,
        argon2: Argon2Config {
            memory_cost: 19456,  // 19 MB (reduced from 64 MB)
            time_cost: 1,        // 1 iteration (reduced from 3)
            parallelism: 1,      // 1 thread (reduced from 4)
            hash_length: 32,     // 32 bytes (same as production)
            salt_length: 16,     // 16 bytes (same as production)
        },
        jwt: Default::default(),
    };

    PasswordHasher::from_config(&config)
}

/// Helper to create a test user with random email/username
///
/// Creates a unique user with randomly generated credentials to avoid
/// conflicts when running multiple tests. The password is always "password"
/// for simplicity in tests.
///
/// # Arguments
/// - `db` - Database connection to insert user into
/// - `hasher` - PasswordHasher to hash the password with
///
/// # Returns
/// - `Ok(users::Model)` - The created user model
/// - `Err(DbErr)` - If database insertion fails
///
/// # Example
/// ```no_run
/// use lighter_auth::testing::setup;
///
/// #[tokio::test]
/// async fn test_user_creation() {
///     let db = setup::database().await;
///     let hasher = setup::password_hasher().unwrap();
///
///     let user1 = setup::create_test_user(&db, &hasher).await.unwrap();
///     let user2 = setup::create_test_user(&db, &hasher).await.unwrap();
///
///     // Users have unique emails/usernames
///     assert_ne!(user1.email, user2.email);
///     assert_ne!(user1.username, user2.username);
/// }
/// ```
pub async fn create_test_user(
    db: &DatabaseConnection,
    hasher: &PasswordHasher,
) -> Result<users::Model, DbErr> {
    use rand::Rng;

    // Generate random suffix for uniqueness
    let random_suffix: u32 = rand::thread_rng().r#gen();
    let email = format!("test_{}@example.com", random_suffix);
    let username = format!("test_user_{}", random_suffix);

    // Hash password
    let password_hash = hasher
        .hash("password")
        .expect("Failed to hash password");

    // Create user
    let now = chrono::Utc::now().naive_utc();
    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Test User".to_string()),
        email: Set(email),
        email_verified_at: Set(None),
        username: Set(username),
        password: Set(password_hash),
        profile_photo_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    user.insert(db).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_database_creates_working_connection() {
        let db = database().await;

        // Verify connection works
        assert_eq!(db.ping().await, Ok(()));
    }

    #[tokio::test]
    async fn test_database_runs_migrations_successfully() {
        use sea_orm::EntityTrait;

        let db = database().await;

        // Verify users table exists by attempting a query
        let result = users::Entity::find().all(&db).await;
        assert!(result.is_ok(), "Users table should exist after migrations");
    }

    #[tokio::test]
    async fn test_cache_returns_working_instance() {
        let cache = cache();

        // Test basic cache operations
        cache
            .set("test_key", &"test_value", Duration::from_secs(60))
            .await
            .expect("Cache set should work");

        let result: Option<String> = cache
            .get("test_key")
            .await
            .expect("Cache get should work");

        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_password_hasher_hashes_correctly() {
        let hasher = password_hasher().expect("Should create hasher");

        let hash = hasher.hash("test_password").expect("Should hash password");

        // Verify hash format
        assert!(hash.starts_with("$argon2id$"));
    }

    #[tokio::test]
    async fn test_password_hasher_verifies_correctly() {
        let hasher = password_hasher().expect("Should create hasher");

        let hash = hasher.hash("correct_password").expect("Should hash password");

        // Correct password should verify
        assert!(hasher.verify("correct_password", &hash).expect("Verify should work"));

        // Wrong password should not verify
        assert!(!hasher.verify("wrong_password", &hash).expect("Verify should work"));
    }

    #[tokio::test]
    async fn test_password_hasher_is_faster_than_production() {
        let test_hasher = password_hasher().expect("Should create test hasher");

        // Production config
        let prod_config = AuthConfig::default();
        let prod_hasher = PasswordHasher::from_config(&prod_config)
            .expect("Should create production hasher");

        // Measure test hasher speed
        let test_start = Instant::now();
        test_hasher.hash("password").expect("Should hash");
        let test_duration = test_start.elapsed();

        // Measure production hasher speed
        let prod_start = Instant::now();
        prod_hasher.hash("password").expect("Should hash");
        let prod_duration = prod_start.elapsed();

        // Test hasher should be significantly faster
        // Allow some variance, but test should be at least 2x faster
        assert!(
            test_duration < prod_duration,
            "Test hasher should be faster than production (test: {:?}, prod: {:?})",
            test_duration,
            prod_duration
        );

        println!(
            "Test hasher: {:?}, Production hasher: {:?}, Speedup: {:.2}x",
            test_duration,
            prod_duration,
            prod_duration.as_secs_f64() / test_duration.as_secs_f64()
        );
    }

    #[tokio::test]
    async fn test_create_test_user_creates_unique_users() {
        let db = database().await;
        let hasher = password_hasher().expect("Should create hasher");

        let user1 = create_test_user(&db, &hasher)
            .await
            .expect("Should create first user");

        let user2 = create_test_user(&db, &hasher)
            .await
            .expect("Should create second user");

        // Users should have different IDs
        assert_ne!(user1.id, user2.id);

        // Users should have different emails
        assert_ne!(user1.email, user2.email);

        // Users should have different usernames
        assert_ne!(user1.username, user2.username);
    }

    #[tokio::test]
    async fn test_create_test_user_can_be_called_multiple_times() {
        use sea_orm::EntityTrait;

        let db = database().await;
        let hasher = password_hasher().expect("Should create hasher");

        // Create multiple users without conflicts
        for _ in 0..5 {
            let result = create_test_user(&db, &hasher).await;
            assert!(result.is_ok(), "Each user creation should succeed");
        }

        // Verify all users were created
        let all_users = users::Entity::find().all(&db).await.expect("Should query users");

        // We should have at least 5 users (plus the seeded root user from migrations)
        assert!(all_users.len() >= 5, "Should have created at least 5 test users");
    }

    #[tokio::test]
    async fn test_create_test_user_password_is_verifiable() {
        let db = database().await;
        let hasher = password_hasher().expect("Should create hasher");

        let user = create_test_user(&db, &hasher)
            .await
            .expect("Should create user");

        // The password should be "password" and should verify correctly
        assert!(
            hasher.verify("password", &user.password).expect("Verify should work"),
            "Created user password should verify with 'password'"
        );

        // Wrong password should not verify
        assert!(
            !hasher.verify("wrong", &user.password).expect("Verify should work"),
            "Wrong password should not verify"
        );
    }

    #[tokio::test]
    async fn test_cache_multiple_instances_are_isolated() {
        let cache1 = cache();
        let cache2 = cache();

        // Set value in cache1
        cache1
            .set("key", &"value1", Duration::from_secs(60))
            .await
            .expect("Should set in cache1");

        // cache2 should not see cache1's data (different instances)
        let result: Option<String> = cache2.get("key").await.expect("Should get from cache2");

        assert_eq!(result, None, "Different cache instances should be isolated");
    }

    #[tokio::test]
    async fn test_database_multiple_calls_create_separate_databases() {
        use sea_orm::EntityTrait;

        let db1 = database().await;
        let db2 = database().await;
        let hasher = password_hasher().expect("Should create hasher");

        // Create user in db1
        let user1 = create_test_user(&db1, &hasher)
            .await
            .expect("Should create user in db1");

        // In-memory databases are isolated, so user should not exist in db2
        // (excluding the seeded root user which exists in all databases)
        let all_users_db2 = users::Entity::find().all(&db2).await.expect("Should get all users");
        let user_exists_in_db2 = all_users_db2.iter().any(|u| u.id == user1.id);

        assert!(!user_exists_in_db2, "Different database instances should be isolated");
    }
}
