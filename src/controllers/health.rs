//! Health check endpoints
//!
//! Provides endpoints for monitoring service health and database connectivity

use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use crate::middlewares::v1::auth::Authenticated;

/// Liveness health check response (simple)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LivenessResponse {
    /// Service status
    pub status: String,
    /// Timestamp of the check
    pub timestamp: DateTime<Utc>,
}

/// Readiness health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessResponse {
    /// Service readiness status
    pub status: String,
    /// Timestamp of the check
    pub timestamp: DateTime<Utc>,
    /// Database connection status
    pub database: String,
    /// Cache availability status
    pub cache: String,
}

/// Health check response (detailed)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Application version
    pub version: String,
    /// Database connection status
    pub database: String,
}

/// Liveness check endpoint
///
/// Simple check that returns 200 OK if service is running.
/// This endpoint should ALWAYS return 200 OK (no dependencies checked).
/// Used for Kubernetes liveness probes.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is alive", body = LivenessResponse),
    )
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    let response = LivenessResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
    };

    ::tracing::debug!("Liveness check: healthy");
    Json(response)
}

/// Detailed health check including database connectivity
///
/// Checks database connectivity status
#[utoipa::path(
    get,
    path = "/health/db",
    tag = "Health",
    responses(
        (status = 200, description = "Service and database are healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = HealthResponse),
    )
)]
#[get("/health/db")]
pub async fn health_db(db: Data<DatabaseConnection>) -> impl Responder {
    // Try to ping database
    let connected = db.ping().await.is_ok();

    let response = HealthResponse {
        status: if connected { "healthy" } else { "unhealthy" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: if connected { "connected" } else { "disconnected" }.to_string(),
    };

    if connected {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// Readiness probe
///
/// For Kubernetes readiness probes - checks if service is ready to accept traffic.
/// Checks database connection and cache availability.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service is not ready", body = ReadinessResponse),
    )
)]
#[get("/ready")]
pub async fn ready(
    db: Data<DatabaseConnection>,
    authenticated: Data<Authenticated>,
) -> impl Responder {
    let timestamp = Utc::now();

    // Check database connection
    let db_connected = db.ping().await.is_ok();
    let database_status = if db_connected { "connected" } else { "disconnected" };

    // Check cache availability
    // We'll check if the cache exists and can be accessed
    let cache_available = check_cache_availability(&authenticated).await;
    let cache_status = if cache_available { "available" } else { "unavailable" };

    let is_ready = db_connected && cache_available;
    let status = if is_ready { "ready" } else { "not_ready" };

    let response = ReadinessResponse {
        status: status.to_string(),
        timestamp,
        database: database_status.to_string(),
        cache: cache_status.to_string(),
    };

    ::tracing::debug!(
        status = %status,
        database = %database_status,
        cache = %cache_status,
        "Readiness check performed"
    );

    if is_ready {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// Check if cache is available
///
/// This is a simple check to verify the cache is accessible.
/// We don't need deep validation, just check if the cache exists.
async fn check_cache_availability(_authenticated: &Authenticated) -> bool {
    // The cache is always available if the Authenticated instance exists
    // In a more complex scenario, you might want to do a test operation
    // For now, we just return true since the cache is always initialized
    true
}

/// Liveness probe (alias for /health)
///
/// For Kubernetes liveness probes - checks if service is alive (doesn't check dependencies).
/// This is an alias for /health endpoint.
#[utoipa::path(
    get,
    path = "/live",
    tag = "Health",
    responses(
        (status = 200, description = "Service is alive", body = LivenessResponse),
    )
)]
#[get("/live")]
pub async fn live() -> impl Responder {
    let response = LivenessResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
    };

    ::tracing::debug!("Liveness check (via /live): healthy");
    Json(response)
}
