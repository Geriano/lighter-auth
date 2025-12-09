//! Security test harness for lighter-auth
//!
//! Run with: cargo test --features sqlite security
//!
//! This test suite covers:
//! - SQL injection prevention across all endpoints
//! - Authentication security
//! - Input validation and sanitization
//! - Database integrity verification

mod security;
