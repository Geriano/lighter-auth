//! Database pool with circuit breaker integration
//!
//! This module provides a DatabasePool wrapper that protects database operations
//! with a circuit breaker pattern, preventing cascade failures when the database
//! becomes unavailable or experiences issues.
//!
//! # Example
//!
//! ```rust
//! use lighter_auth::database::DatabasePool;
//! use lighter_auth::config::ResilienceConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let db_conn = sea_orm::Database::connect("sqlite::memory:").await?;
//! let config = ResilienceConfig::default();
//!
//! let pool = DatabasePool::new(db_conn, config);
//!
//! // Execute query with circuit breaker protection
//! let result = pool.execute(|db| {
//!     Box::pin(async move {
//!         // Your database query here
//!         Ok(())
//!     })
//! }).await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use lighter_common::errors::DatabaseError as CommonDatabaseError;
use thiserror::Error;

use crate::config::ResilienceConfig;
use crate::resilience::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};

/// Database operation error with circuit breaker support
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Circuit breaker is open
    #[error("Database circuit breaker is open, service temporarily unavailable")]
    CircuitOpen,

    /// Database operation failed
    #[error("Database operation failed: {0}")]
    QueryFailed(#[from] DbErr),

    /// Connection error
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

impl From<CircuitBreakerError<DbErr>> for DatabaseError {
    fn from(error: CircuitBreakerError<DbErr>) -> Self {
        match error {
            CircuitBreakerError::Open { .. } => DatabaseError::CircuitOpen,
            CircuitBreakerError::Inner(err) => DatabaseError::QueryFailed(err),
        }
    }
}

/// Convert to lighter-common DatabaseError for HTTP error handling
impl From<DatabaseError> for CommonDatabaseError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::CircuitOpen => {
                // Return PoolError as the circuit breaker is part of the pool
                CommonDatabaseError::PoolError(
                    "Database temporarily unavailable due to circuit breaker".to_string()
                )
            }
            DatabaseError::QueryFailed(err) => {
                // Convert SeaORM DbErr to CommonDatabaseError
                CommonDatabaseError::QueryError(err.to_string())
            }
            DatabaseError::ConnectionError(msg) => CommonDatabaseError::ConnectionError(msg),
            DatabaseError::TransactionError(msg) => CommonDatabaseError::TransactionError(msg),
        }
    }
}

/// Database pool with circuit breaker protection
///
/// This wrapper provides resilient database access by:
/// - Protecting against cascade failures with circuit breaker
/// - Logging all state transitions
/// - Providing metrics for monitoring
/// - Supporting both single queries and transactions
///
/// # Thread Safety
///
/// DatabasePool is thread-safe and can be cloned to share across threads.
/// The underlying CircuitBreaker uses atomic operations and mutexes for
/// safe concurrent access.
#[derive(Clone)]
pub struct DatabasePool {
    /// Underlying database connection
    connection: Arc<DatabaseConnection>,
    /// Circuit breaker for fault tolerance
    circuit_breaker: Arc<CircuitBreaker>,
    /// Resilience configuration
    config: ResilienceConfig,
}

impl DatabasePool {
    /// Create a new DatabasePool with circuit breaker protection
    ///
    /// # Arguments
    ///
    /// * `connection` - SeaORM database connection
    /// * `config` - Resilience configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::database::DatabasePool;
    /// use lighter_auth::config::ResilienceConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = sea_orm::Database::connect("sqlite::memory:").await?;
    /// let config = ResilienceConfig::default();
    /// let pool = DatabasePool::new(db, config);
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(connection, config), fields(
        circuit_breaker_enabled = %config.circuit_breaker.enabled,
        failure_threshold = %config.circuit_breaker.threshold,
        timeout = %config.circuit_breaker.timeout
    ))]
    pub fn new(connection: DatabaseConnection, config: ResilienceConfig) -> Self {
        let circuit_breaker = if config.circuit_breaker.enabled {
            tracing::info!(
                threshold = config.circuit_breaker.threshold,
                timeout = config.circuit_breaker.timeout,
                success_threshold = config.circuit_breaker.success_threshold,
                "Initializing database circuit breaker"
            );

            let cb_config = CircuitBreakerConfig {
                failure_threshold: config.circuit_breaker.threshold,
                success_threshold: config.circuit_breaker.success_threshold,
                timeout: config.circuit_breaker.timeout,
            };

            CircuitBreaker::with_config("database".to_string(), cb_config)
        } else {
            tracing::warn!("Database circuit breaker is disabled");
            CircuitBreaker::new("database-disabled".to_string())
        };

        Self {
            connection: Arc::new(connection),
            circuit_breaker: Arc::new(circuit_breaker),
            config,
        }
    }

    /// Get a reference to the underlying database connection
    ///
    /// # Warning
    ///
    /// Using this method bypasses circuit breaker protection.
    /// Only use for read-only operations or when you need direct access.
    /// For write operations, prefer using `execute` or `transaction`.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Execute a database operation with circuit breaker protection
    ///
    /// # Arguments
    ///
    /// * `f` - Async function that takes a database connection and returns a Result
    ///
    /// # Returns
    ///
    /// Result containing the operation result or DatabaseError
    ///
    /// # Circuit Breaker Behavior
    ///
    /// - **Closed State**: Operation executes normally
    /// - **Open State**: Operation is rejected immediately with CircuitOpen error
    /// - **HalfOpen State**: Operation executes as a test; success may close circuit
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::database::DatabasePool;
    /// # use sea_orm::{EntityTrait, DbErr};
    ///
    /// # async fn example(pool: DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = pool.execute(|db| {
    ///     Box::pin(async move {
    ///         // Your query here
    ///         Ok::<_, DbErr>(42)
    ///     })
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, f), fields(
        circuit_state = %self.circuit_breaker.state(),
        total_calls = %self.circuit_breaker.total_calls(),
        failure_rate = %self.circuit_breaker.failure_rate()
    ))]
    pub async fn execute<F, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&DatabaseConnection) -> Pin<Box<dyn Future<Output = Result<T, DbErr>> + Send>>
            + Send,
        T: Send,
    {
        if !self.config.circuit_breaker.enabled {
            // Circuit breaker disabled, execute directly
            return f(&self.connection).await.map_err(DatabaseError::from);
        }

        // Execute with circuit breaker protection
        let db = Arc::clone(&self.connection);
        let result = self
            .circuit_breaker
            .call(async move {
                let db_ref: &DatabaseConnection = &db;
                f(db_ref).await
            })
            .await;

        match result {
            Ok(value) => {
                tracing::debug!("Database operation succeeded");
                Ok(value)
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    circuit_state = %self.circuit_breaker.state(),
                    "Database operation failed"
                );
                Err(DatabaseError::from(e))
            }
        }
    }

    /// Execute a database transaction with circuit breaker protection
    ///
    /// # Arguments
    ///
    /// * `f` - Async function that takes a transaction and returns a Result
    ///
    /// # Behavior
    ///
    /// - Begins a transaction
    /// - Executes the provided function with circuit breaker protection
    /// - Commits on success
    /// - Rolls back on failure
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::database::DatabasePool;
    /// # use sea_orm::{DatabaseTransaction, DbErr};
    ///
    /// # async fn example(pool: DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    /// pool.transaction(|txn| {
    ///     Box::pin(async move {
    ///         // Multiple operations in transaction
    ///         Ok::<_, DbErr>(())
    ///     })
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, f), fields(
        circuit_state = %self.circuit_breaker.state()
    ))]
    pub async fn transaction<F, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(
                &sea_orm::DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, DbErr>> + Send>>
            + Send,
        T: Send,
    {
        if !self.config.circuit_breaker.enabled {
            // Circuit breaker disabled, execute directly
            let txn = self
                .connection
                .begin()
                .await
                .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

            let result = f(&txn).await.map_err(DatabaseError::from)?;

            txn.commit()
                .await
                .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

            return Ok(result);
        }

        // Execute transaction with circuit breaker protection
        let db = Arc::clone(&self.connection);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        circuit_breaker
            .call(async move {
                let txn = db
                    .begin()
                    .await
                    .map_err(|e| DbErr::Custom(format!("Failed to begin transaction: {}", e)))?;

                let result = f(&txn).await?;

                txn.commit()
                    .await
                    .map_err(|e| DbErr::Custom(format!("Failed to commit transaction: {}", e)))?;

                tracing::debug!("Transaction committed successfully");
                Ok(result)
            })
            .await
            .map_err(DatabaseError::from)
    }

    /// Get circuit breaker statistics
    ///
    /// # Returns
    ///
    /// Tuple of (total_calls, total_failures, failure_rate, state)
    pub fn stats(&self) -> (u64, u64, f64, String) {
        (
            self.circuit_breaker.total_calls(),
            self.circuit_breaker.total_failures(),
            self.circuit_breaker.failure_rate(),
            self.circuit_breaker.state().to_string(),
        )
    }

    /// Reset the circuit breaker to closed state
    ///
    /// # Warning
    ///
    /// This should only be used for testing or administrative purposes.
    /// In production, let the circuit breaker manage state transitions automatically.
    pub fn reset_circuit_breaker(&self) {
        tracing::warn!("Manually resetting database circuit breaker");
        self.circuit_breaker.reset();
    }

    /// Check if circuit breaker is enabled
    pub fn is_circuit_breaker_enabled(&self) -> bool {
        self.config.circuit_breaker.enabled
    }
}

impl std::fmt::Debug for DatabasePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabasePool")
            .field("circuit_breaker", &self.circuit_breaker)
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CircuitBreakerConfig as ConfigCBConfig;
    use sea_orm::Database;

    async fn create_test_pool() -> DatabasePool {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        let config = ResilienceConfig {
            circuit_breaker: ConfigCBConfig {
                enabled: true,
                threshold: 3,
                timeout: 2,
                success_threshold: 2,
            },
            retry: crate::config::RetryConfig::default(),
        };

        DatabasePool::new(db, config)
    }

    #[tokio::test]
    async fn test_database_pool_creation() {
        let pool = create_test_pool().await;
        assert!(pool.is_circuit_breaker_enabled());

        let (total_calls, total_failures, failure_rate, state) = pool.stats();
        assert_eq!(total_calls, 0);
        assert_eq!(total_failures, 0);
        assert_eq!(failure_rate, 0.0);
        assert_eq!(state, "Closed");
    }

    #[tokio::test]
    async fn test_successful_query() {
        let pool = create_test_pool().await;

        // Direct access test
        let db = pool.connection();
        let result = db.ping().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create database");

        let config = ResilienceConfig {
            circuit_breaker: ConfigCBConfig {
                enabled: false,
                threshold: 3,
                timeout: 60,
                success_threshold: 2,
            },
            retry: crate::config::RetryConfig::default(),
        };

        let pool = DatabasePool::new(db, config);

        // Circuit breaker should be disabled
        assert!(!pool.is_circuit_breaker_enabled());
    }

    #[tokio::test]
    async fn test_reset_circuit_breaker() {
        let pool = create_test_pool().await;

        // Reset circuit (should be Closed initially)
        pool.reset_circuit_breaker();

        let (_, _, _, state) = pool.stats();
        assert_eq!(state, "Closed");
    }
}
