//! Security tests module
//!
//! This module contains security-focused tests including:
//! - SQL injection prevention
//! - XSS (Cross-Site Scripting) prevention
//! - Authentication bypass attempts
//! - Input validation security
//!
//! Run with: cargo test --features sqlite security

pub mod sql_injection_test;
pub mod xss_test;
