//! Unit test harness for lighter-auth
//!
//! Run with: cargo test --features sqlite unit
//!
//! This test suite covers:
//! - Configuration loading from default.toml
//! - Environment-specific configuration overrides (dev, staging, prod)
//! - Environment variable override precedence
//! - Configuration precedence order (defaults < files < env vars)
//! - Configuration validation for all modules
//! - Invalid value detection and error messages
//! - TLS configuration validation
//! - Argon2 password hashing parameters
//! - Metrics histogram buckets validation
//! - Circuit breaker and retry configuration
//! - Rate limiting configuration
//! - Complete configuration loading flow

mod unit;
