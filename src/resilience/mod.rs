//! Resilience patterns for fault-tolerant microservices
//!
//! This module provides implementations of resilience patterns that help build
//! robust and fault-tolerant distributed systems.
//!
//! # Available Patterns
//!
//! - **Circuit Breaker**: Prevents cascading failures by temporarily blocking
//!   requests to failing services, giving them time to recover.
//!
//! # Example
//!
//! ```rust
//! use lighter_auth::resilience::{CircuitBreaker, CircuitBreakerConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a circuit breaker for an external service
//! let config = CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     success_threshold: 2,
//!     timeout: 60,
//! };
//!
//! let cb = CircuitBreaker::with_config("payment-api".to_string(), config);
//!
//! // Use it to protect your calls
//! let result = cb.call(async {
//!     // Your risky operation here
//!     Ok::<_, std::io::Error>(())
//! }).await;
//! # Ok(())
//! # }
//! ```

mod circuit_breaker;

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState,
};
