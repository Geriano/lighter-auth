//! Health check endpoints
//!
//! Provides endpoints for monitoring service health and database connectivity

use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::database::DatabasePool;

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Application version
    pub version: String,
    /// Database status
    pub database: DatabaseHealthStatus,
}

/// Database health status including circuit breaker information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseHealthStatus {
    /// Database connection status
    pub connected: bool,
    /// Circuit breaker state (Closed, Open, HalfOpen)
    pub circuit_breaker_state: String,
    /// Circuit breaker enabled
    pub circuit_breaker_enabled: bool,
    /// Total database calls
    pub total_calls: u64,
    /// Total failed calls
    pub total_failures: u64,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
}

/// Basic health check
///
/// Returns service status without checking dependencies
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
    )
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealthStatus {
            connected: false,
            circuit_breaker_state: "unknown".to_string(),
            circuit_breaker_enabled: false,
            total_calls: 0,
            total_failures: 0,
            failure_rate: 0.0,
        },
    };

    Json(response)
}

/// Detailed health check including database and circuit breaker status
///
/// Checks database connectivity and returns circuit breaker metrics
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
pub async fn health_db(pool: Data<DatabasePool>) -> impl Responder {
    let (total_calls, total_failures, failure_rate, state) = pool.stats();

    // Try to ping database
    let db_ref = pool.connection();
    let connected = db_ref.ping().await.is_ok();

    let response = HealthResponse {
        status: if connected { "healthy" } else { "unhealthy" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealthStatus {
            connected,
            circuit_breaker_state: state.clone(),
            circuit_breaker_enabled: pool.is_circuit_breaker_enabled(),
            total_calls,
            total_failures,
            failure_rate,
        },
    };

    if connected {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// Readiness probe
///
/// For Kubernetes readiness probes - checks if service is ready to accept traffic
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready"),
    )
)]
#[get("/ready")]
pub async fn ready(pool: Data<DatabasePool>) -> impl Responder {
    // Check if database is accessible
    let db_ref = pool.connection();
    let is_ready = db_ref.ping().await.is_ok();

    if is_ready {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "reason": "database_unavailable"
        }))
    }
}

/// Liveness probe
///
/// For Kubernetes liveness probes - checks if service is alive (doesn't check dependencies)
#[utoipa::path(
    get,
    path = "/live",
    tag = "Health",
    responses(
        (status = 200, description = "Service is alive"),
    )
)]
#[get("/live")]
pub async fn live() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive"
    }))
}
