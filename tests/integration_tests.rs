//! Integration test harness for lighter-auth
//!
//! Run with: cargo test --features sqlite integration
//!
//! This test suite covers:
//! - Complete authentication flow (login, get user, logout)
//! - User creation and management
//! - Password updates
//! - Token validation and invalidation
//! - Error handling for invalid requests

mod integration;
