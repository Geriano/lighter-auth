//! Security tests module
//!
//! This module contains security-focused tests including:
//! - SQL injection prevention
//! - Authentication bypass attempts
//! - Input validation security
//!
//! Run with: cargo test --features sqlite security

pub mod sql_injection_test;
