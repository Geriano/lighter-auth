#![deny(warnings)]

#[macro_use]
extern crate actix_web;

// Re-export all public modules
pub mod api;
pub mod cache;
pub mod config;
pub mod controllers;
pub mod database;
pub mod entities;
pub mod metrics;
pub mod middlewares;
pub mod models;
pub mod requests;
pub mod resilience;
pub mod responses;
pub mod router;
pub mod security;
pub mod services;

// Testing utilities (always available for integration tests)
pub mod testing;

// Re-export commonly used types for convenience
pub use cache::{Cache, CacheKey, CacheStats, HybridCache, LocalCache, NullCache, RedisCache};
pub use database::DatabasePool;
pub use metrics::{AppMetrics, MetricsMiddleware};
pub use middlewares::v1::auth::Authenticated;
pub use security::{RateLimitMiddleware, SecurityHeadersMiddleware};
