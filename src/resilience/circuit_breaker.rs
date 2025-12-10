//! Circuit Breaker Pattern Implementation
//!
//! This module provides a production-ready circuit breaker for resilience against cascading failures.
//! The circuit breaker monitors operation failures and can temporarily block requests to failing services,
//! giving them time to recover.
//!
//! # State Machine
//!
//! ```text
//! ┌─────────┐
//! │ Closed  │ ◄──────────────────┐
//! │ (Normal)│                    │
//! └────┬────┘                    │
//!      │ failure_threshold       │ success_threshold
//!      │ consecutive failures    │ consecutive successes
//!      ▼                         │
//! ┌─────────┐    timeout    ┌────┴──────┐
//! │  Open   │───────────────► HalfOpen  │
//! │(Failing)│                │ (Testing) │
//! └─────────┘◄───────────────└───────────┘
//!                any failure
//! ```
//!
//! # Example
//!
//! ```rust
//! use lighter_auth::resilience::{CircuitBreaker, CircuitBreakerConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a circuit breaker with default config
//! let cb = CircuitBreaker::new("payment-service".to_string());
//!
//! // Or with custom config
//! let config = CircuitBreakerConfig {
//!     failure_threshold: 10,
//!     success_threshold: 3,
//!     timeout: 120,
//! };
//! let cb_custom = CircuitBreaker::with_config("payment-service".to_string(), config);
//!
//! // Use the circuit breaker to protect a call
//! let result = cb.call(async {
//!     // Make your risky call here (e.g., HTTP request, database query, etc.)
//!     Ok::<String, std::io::Error>("Success".to_string())
//! }).await;
//!
//! match result {
//!     Ok(response) => println!("Success: {:?}", response),
//!     Err(e) => println!("Failed: {:?}", e),
//! }
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation, allowing all requests through
    Closed,
    /// Failing state, rejecting all requests until timeout expires
    Open,
    /// Testing state, allowing limited requests to check if service recovered
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Thread-safe statistics tracking for circuit breaker
#[derive(Debug)]
pub struct CircuitBreakerStats {
    /// Number of consecutive failures
    consecutive_failures: AtomicU64,
    /// Number of consecutive successes
    consecutive_successes: AtomicU64,
    /// Total number of calls attempted
    total_calls: AtomicU64,
    /// Total number of failed calls
    total_failures: AtomicU64,
    /// Unix timestamp of last failure (seconds)
    last_failure_time: AtomicU64,
}

impl CircuitBreakerStats {
    /// Create new statistics tracker
    fn new() -> Self {
        Self {
            consecutive_failures: AtomicU64::new(0),
            consecutive_successes: AtomicU64::new(0),
            total_calls: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
        }
    }

    /// Record a successful call
    fn record_success(&self) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        self.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// Record a failed call
    fn record_failure(&self) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);

        // Update last failure time to current unix timestamp
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.last_failure_time
                .store(duration.as_secs(), Ordering::Relaxed);
        }
    }

    /// Get consecutive failures count
    fn consecutive_failures(&self) -> u64 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// Get consecutive successes count
    fn consecutive_successes(&self) -> u64 {
        self.consecutive_successes.load(Ordering::Relaxed)
    }

    /// Get total calls count
    pub fn total_calls(&self) -> u64 {
        self.total_calls.load(Ordering::Relaxed)
    }

    /// Get total failures count
    pub fn total_failures(&self) -> u64 {
        self.total_failures.load(Ordering::Relaxed)
    }

    /// Get seconds elapsed since last failure
    fn seconds_since_last_failure(&self) -> u64 {
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        if last_failure == 0 {
            return u64::MAX; // Never failed
        }

        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let now = duration.as_secs();
            now.saturating_sub(last_failure)
        } else {
            0
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,
    /// Number of consecutive successes before closing circuit from half-open state
    pub success_threshold: u32,
    /// Timeout in seconds before transitioning from Open to HalfOpen
    pub timeout: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: 60,
        }
    }
}

/// Circuit breaker error
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open, rejecting requests
    #[error("Circuit breaker is open for {name}")]
    Open { name: String },
    /// The underlying operation failed
    #[error("Operation failed: {0}")]
    Inner(#[source] E),
}

/// Production-ready circuit breaker implementation
///
/// The circuit breaker protects your application from cascading failures by monitoring
/// the success/failure rate of operations and temporarily blocking requests when failures
/// exceed a threshold.
///
/// # Thread Safety
///
/// This implementation is fully thread-safe using atomic operations for statistics
/// and a mutex for state transitions. It can be safely cloned and shared across
/// multiple threads or async tasks.
///
/// # Performance
///
/// The circuit breaker adds minimal overhead (< 1% in most cases):
/// - Atomic reads/writes for statistics: ~1-5ns
/// - State check with possible transition: ~10-50ns
/// - Mutex contention only during state transitions (rare)
#[derive(Clone)]
pub struct CircuitBreaker {
    /// Name for logging and debugging
    name: String,
    /// Current circuit state
    state: Arc<Mutex<CircuitState>>,
    /// Statistics tracker
    stats: Arc<CircuitBreakerStats>,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::resilience::CircuitBreaker;
    ///
    /// let cb = CircuitBreaker::new("database".to_string());
    /// ```
    pub fn new(name: String) -> Self {
        Self::with_config(name, CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::resilience::{CircuitBreaker, CircuitBreakerConfig};
    ///
    /// let config = CircuitBreakerConfig {
    ///     failure_threshold: 10,
    ///     success_threshold: 3,
    ///     timeout: 120,
    /// };
    /// let cb = CircuitBreaker::with_config("api".to_string(), config);
    /// ```
    pub fn with_config(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            stats: Arc::new(CircuitBreakerStats::new()),
            config,
        }
    }

    /// Get the current state of the circuit breaker
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::resilience::{CircuitBreaker, CircuitState};
    ///
    /// let cb = CircuitBreaker::new("service".to_string());
    /// assert_eq!(cb.state(), CircuitState::Closed);
    /// ```
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Get the circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get total number of calls
    pub fn total_calls(&self) -> u64 {
        self.stats.total_calls()
    }

    /// Get total number of failures
    pub fn total_failures(&self) -> u64 {
        self.stats.total_failures()
    }

    /// Get failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f64 {
        let total = self.stats.total_calls();
        if total == 0 {
            return 0.0;
        }
        self.stats.total_failures() as f64 / total as f64
    }

    /// Execute an operation protected by the circuit breaker
    ///
    /// # State Transitions
    ///
    /// - **Closed → Open**: When consecutive_failures >= failure_threshold
    /// - **Open → HalfOpen**: When timeout seconds have elapsed since last_failure_time
    /// - **HalfOpen → Closed**: When consecutive_successes >= success_threshold
    /// - **HalfOpen → Open**: On any failure
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::resilience::CircuitBreaker;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cb = CircuitBreaker::new("external-api".to_string());
    ///
    /// let result = cb.call(async {
    ///     // Your potentially failing operation
    ///     Ok::<_, std::io::Error>(42)
    /// }).await;
    ///
    /// match result {
    ///     Ok(value) => println!("Got: {}", value),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check current state and handle transitions
        let current_state = {
            let mut state = self.state.lock().unwrap();

            match *state {
                CircuitState::Open => {
                    // Check if timeout has elapsed
                    let elapsed = self.stats.seconds_since_last_failure();
                    if elapsed >= self.config.timeout {
                        // Transition to HalfOpen
                        *state = CircuitState::HalfOpen;
                        tracing::info!(
                            circuit_breaker = %self.name,
                            state = "Open -> HalfOpen",
                            elapsed_seconds = elapsed,
                            "Circuit breaker transitioning to HalfOpen"
                        );
                        CircuitState::HalfOpen
                    } else {
                        // Still open, reject request
                        return Err(CircuitBreakerError::Open {
                            name: self.name.clone(),
                        });
                    }
                }
                state => state,
            }
        };

        // Execute the operation
        match f.await {
            Ok(result) => {
                // Record success
                self.stats.record_success();

                // Check for HalfOpen → Closed transition
                if current_state == CircuitState::HalfOpen
                    && self.stats.consecutive_successes() >= self.config.success_threshold as u64 {
                        let mut state = self.state.lock().unwrap();
                        if *state == CircuitState::HalfOpen {
                            *state = CircuitState::Closed;
                            tracing::info!(
                                circuit_breaker = %self.name,
                                state = "HalfOpen -> Closed",
                                consecutive_successes = self.stats.consecutive_successes(),
                                "Circuit breaker closed after successful recovery"
                            );
                        }
                    }

                Ok(result)
            }
            Err(err) => {
                // Record failure
                self.stats.record_failure();

                // Check for state transitions
                let mut state = self.state.lock().unwrap();

                match *state {
                    CircuitState::Closed => {
                        // Check for Closed → Open transition
                        if self.stats.consecutive_failures() >= self.config.failure_threshold as u64
                        {
                            *state = CircuitState::Open;
                            tracing::warn!(
                                circuit_breaker = %self.name,
                                state = "Closed -> Open",
                                consecutive_failures = self.stats.consecutive_failures(),
                                failure_threshold = self.config.failure_threshold,
                                "Circuit breaker opened due to consecutive failures"
                            );
                        }
                    }
                    CircuitState::HalfOpen => {
                        // Any failure in HalfOpen → Open
                        *state = CircuitState::Open;
                        tracing::warn!(
                            circuit_breaker = %self.name,
                            state = "HalfOpen -> Open",
                            "Circuit breaker re-opened after failure in HalfOpen state"
                        );
                    }
                    CircuitState::Open => {
                        // Already open, shouldn't reach here but handle gracefully
                    }
                }

                Err(CircuitBreakerError::Inner(err))
            }
        }
    }

    /// Manually reset the circuit breaker to Closed state
    ///
    /// This is useful for testing or administrative purposes.
    /// In production, prefer letting the circuit breaker manage state automatically.
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        // Note: We don't reset stats, only state
        tracing::info!(
            circuit_breaker = %self.name,
            "Circuit breaker manually reset to Closed"
        );
    }
}

impl fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("name", &self.name)
            .field("state", &self.state())
            .field("consecutive_failures", &self.stats.consecutive_failures())
            .field("consecutive_successes", &self.stats.consecutive_successes())
            .field("total_calls", &self.stats.total_calls())
            .field("total_failures", &self.stats.total_failures())
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
    use std::time::Instant;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "test error")
        }
    }

    impl std::error::Error for TestError {}

    #[tokio::test]
    async fn test_initial_state_is_closed() {
        let cb = CircuitBreaker::new("test".to_string());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_successful_call() {
        let cb = CircuitBreaker::new("test".to_string());

        let result = cb.call(async { Ok::<i32, TestError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.total_calls(), 1);
        assert_eq!(cb.total_failures(), 0);
    }

    #[tokio::test]
    async fn test_failed_call() {
        let cb = CircuitBreaker::new("test".to_string());

        let result = cb.call(async { Err::<i32, _>(TestError) }).await;

        assert!(result.is_err());
        assert_eq!(cb.state(), CircuitState::Closed); // Should still be closed (below threshold)
        assert_eq!(cb.total_calls(), 1);
        assert_eq!(cb.total_failures(), 1);
    }

    #[tokio::test]
    async fn test_closed_to_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: 60,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Fail 3 times to exceed threshold
        for _ in 0..3 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);
        assert_eq!(cb.total_failures(), 3);
    }

    #[tokio::test]
    async fn test_open_state_rejects_calls() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 60,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open state
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Next call should be rejected without executing
        let result = cb.call(async { Ok::<i32, TestError>(42) }).await;

        assert!(matches!(result, Err(CircuitBreakerError::Open { .. })));
        // Total calls should be 2 (the ones that caused open), not 3
        assert_eq!(cb.total_calls(), 2);
    }

    #[tokio::test]
    async fn test_open_to_halfopen_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 1, // 1 second timeout
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open state
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        sleep(TokioDuration::from_secs(2)).await;

        // Next call should transition to HalfOpen and execute
        let result = cb.call(async { Ok::<i32, TestError>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::HalfOpen); // Not closed yet, need more successes
    }

    #[tokio::test]
    async fn test_halfopen_to_closed_after_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 1,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open state
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        // Wait and transition to HalfOpen with first success
        sleep(TokioDuration::from_secs(2)).await;
        let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;

        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Second success should close the circuit
        let _ = cb.call(async { Ok::<i32, TestError>(2) }).await;

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_halfopen_to_open_on_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 1,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open state
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        // Wait and transition to HalfOpen
        sleep(TokioDuration::from_secs(2)).await;
        let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;

        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Failure should reopen circuit
        let _ = cb.call(async { Err::<i32, _>(TestError) }).await;

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_consecutive_failures_reset_on_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: 60,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // 2 failures (below threshold)
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);

        // Success resets consecutive failures
        let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;

        // 2 more failures (should not open because counter was reset)
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let cb = CircuitBreaker::new("test".to_string());

        // 3 successes
        for _ in 0..3 {
            let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;
        }

        // 2 failures
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.total_calls(), 5);
        assert_eq!(cb.total_failures(), 2);
        assert_eq!(cb.failure_rate(), 0.4); // 2/5 = 0.4
    }

    #[tokio::test]
    async fn test_concurrent_calls() {
        let cb = CircuitBreaker::new("test".to_string());
        let cb_clone1 = cb.clone();
        let cb_clone2 = cb.clone();

        let counter = Arc::new(AtomicU32::new(0));
        let counter1 = counter.clone();
        let counter2 = counter.clone();

        // Spawn two tasks that make calls concurrently
        let handle1 = tokio::spawn(async move {
            for _ in 0..100 {
                let c = counter1.clone();
                let _ = cb_clone1
                    .call(async move {
                        c.fetch_add(1, AtomicOrdering::Relaxed);
                        Ok::<_, TestError>(())
                    })
                    .await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for _ in 0..100 {
                let c = counter2.clone();
                let _ = cb_clone2
                    .call(async move {
                        c.fetch_add(1, AtomicOrdering::Relaxed);
                        Ok::<_, TestError>(())
                    })
                    .await;
            }
        });

        let _ = tokio::join!(handle1, handle2);

        assert_eq!(cb.total_calls(), 200);
        assert_eq!(counter.load(AtomicOrdering::Relaxed), 200);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 60,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open state
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }

        assert_eq!(cb.state(), CircuitState::Open);

        // Manually reset
        cb.reset();

        assert_eq!(cb.state(), CircuitState::Closed);

        // Should accept calls again
        let result = cb.call(async { Ok::<i32, TestError>(42) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_transitions_sequence() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: 1,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Start: Closed
        assert_eq!(cb.state(), CircuitState::Closed);

        // Closed -> Open
        for _ in 0..2 {
            let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);

        // Open -> HalfOpen
        sleep(TokioDuration::from_secs(2)).await;
        let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // HalfOpen -> Open (failure)
        let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        assert_eq!(cb.state(), CircuitState::Open);

        // Open -> HalfOpen -> Closed (successes)
        sleep(TokioDuration::from_secs(2)).await;
        let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        let _ = cb.call(async { Ok::<i32, TestError>(2) }).await;
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_success_in_closed_does_not_transition() {
        let cb = CircuitBreaker::new("test".to_string());

        // Many successes in Closed state
        for _ in 0..100 {
            let _ = cb.call(async { Ok::<i32, TestError>(1) }).await;
        }

        // Should still be closed
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_timeout_calculation() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: 2,
        };
        let cb = CircuitBreaker::with_config("test".to_string(), config);

        // Trigger open
        let _ = cb.call(async { Err::<i32, _>(TestError) }).await;
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait 1 second (less than timeout)
        sleep(TokioDuration::from_secs(1)).await;

        // Should still reject
        let result = cb.call(async { Ok::<i32, TestError>(1) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open { .. })));

        // Wait another 2 seconds (total > timeout)
        sleep(TokioDuration::from_secs(2)).await;

        // Should transition to HalfOpen and execute
        let result = cb.call(async { Ok::<i32, TestError>(1) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed); // success_threshold = 1
    }

    #[tokio::test]
    async fn test_performance_overhead() {
        let cb = CircuitBreaker::new("test".to_string());

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = cb.call(async { Ok::<_, TestError>(()) }).await;
        }
        let elapsed = start.elapsed();

        // Circuit breaker overhead should be minimal
        // 10k operations should complete in reasonable time (< 100ms on modern hardware)
        assert!(
            elapsed < TokioDuration::from_millis(100),
            "Circuit breaker overhead too high: {:?}",
            elapsed
        );

        println!(
            "10k operations completed in {:?} ({:.2}ns per operation)",
            elapsed,
            elapsed.as_nanos() as f64 / 10000.0
        );
    }
}
