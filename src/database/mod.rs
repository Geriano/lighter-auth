//! Database module with circuit breaker integration
//!
//! This module provides a DatabasePool wrapper that integrates circuit breaker
//! pattern for resilient database operations.

mod pool;

pub use pool::{DatabasePool, DatabaseError};
