//! Integration tests for health check endpoints
//!
//! This module tests the health check endpoints:
//! - /health (liveness check)
//! - /ready (readiness check)
//! - /live (liveness check alias)
//! - /health/db (detailed health check with database status)
//!
//! Tests cover both success scenarios and failure scenarios where applicable.

use actix_web::http::StatusCode;
use actix_web::test::{call_service, TestRequest};
use serde_json::Value;

// =============================================================================
// LIVENESS TESTS - /health endpoint
// =============================================================================

/// Test that /health endpoint always returns 200 OK
///
/// This is a liveness check that should ALWAYS return success,
/// regardless of dependency status (database, cache, etc.).
#[actix_web::test]
async fn test_health_endpoint_returns_200_ok() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "/health should always return 200 OK");

    // Verify response structure
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    assert_eq!(body_json["status"], "healthy", "Status should be 'healthy'");
    assert!(body_json["timestamp"].is_string(), "Timestamp should be present");

    // Verify timestamp is in ISO 8601 format
    let timestamp = body_json["timestamp"].as_str().unwrap();
    assert!(timestamp.contains("T"), "Timestamp should be in ISO 8601 format");
}

/// Test that /health endpoint response matches expected schema
#[actix_web::test]
async fn test_health_endpoint_response_schema() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = call_service(&service, req).await;
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    // Verify all required fields are present
    assert!(body_json.get("status").is_some(), "Response should have 'status' field");
    assert!(body_json.get("timestamp").is_some(), "Response should have 'timestamp' field");

    // Verify field types
    assert!(body_json["status"].is_string(), "'status' should be a string");
    assert!(body_json["timestamp"].is_string(), "'timestamp' should be a string");
}

// =============================================================================
// LIVENESS TESTS - /live endpoint (alias)
// =============================================================================

/// Test that /live endpoint (alias for /health) returns 200 OK
#[actix_web::test]
async fn test_live_endpoint_returns_200_ok() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/live")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK, "/live should always return 200 OK");

    // Verify response structure
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    assert_eq!(body_json["status"], "healthy", "Status should be 'healthy'");
    assert!(body_json["timestamp"].is_string(), "Timestamp should be present");
}

// =============================================================================
// READINESS TESTS - /ready endpoint
// =============================================================================

/// Test that /ready endpoint returns 200 OK when dependencies are available
///
/// This test verifies that when both database and cache are available,
/// the readiness endpoint returns a successful response.
#[actix_web::test]
async fn test_ready_endpoint_returns_200_when_dependencies_up() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/ready")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "/ready should return 200 OK when dependencies are available"
    );

    // Verify response structure
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    assert_eq!(body_json["status"], "ready", "Status should be 'ready'");
    assert_eq!(body_json["database"], "connected", "Database should be connected");
    assert_eq!(body_json["cache"], "available", "Cache should be available");
    assert!(body_json["timestamp"].is_string(), "Timestamp should be present");
}

/// Test that /ready endpoint response matches expected schema
#[actix_web::test]
async fn test_ready_endpoint_response_schema() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/ready")
        .to_request();

    let resp = call_service(&service, req).await;
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    // Verify all required fields are present
    assert!(body_json.get("status").is_some(), "Response should have 'status' field");
    assert!(body_json.get("timestamp").is_some(), "Response should have 'timestamp' field");
    assert!(body_json.get("database").is_some(), "Response should have 'database' field");
    assert!(body_json.get("cache").is_some(), "Response should have 'cache' field");

    // Verify field types
    assert!(body_json["status"].is_string(), "'status' should be a string");
    assert!(body_json["timestamp"].is_string(), "'timestamp' should be a string");
    assert!(body_json["database"].is_string(), "'database' should be a string");
    assert!(body_json["cache"].is_string(), "'cache' should be a string");
}

/// Test that /ready endpoint includes both database and cache status
#[actix_web::test]
async fn test_ready_endpoint_checks_all_dependencies() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/ready")
        .to_request();

    let resp = call_service(&service, req).await;
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    // Verify both database and cache are checked
    let database = body_json["database"].as_str().unwrap();
    let cache = body_json["cache"].as_str().unwrap();

    assert!(
        database == "connected" || database == "disconnected",
        "Database status should be either 'connected' or 'disconnected'"
    );
    assert!(
        cache == "available" || cache == "unavailable",
        "Cache status should be either 'available' or 'unavailable'"
    );
}

// =============================================================================
// DETAILED HEALTH TESTS - /health/db endpoint
// =============================================================================

/// Test that /health/db endpoint returns detailed health information
///
/// This endpoint provides comprehensive health information including
/// circuit breaker status and database metrics.
#[actix_web::test]
async fn test_health_db_endpoint_returns_detailed_info() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/health/db")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "/health/db should return 200 OK when database is available"
    );

    // Verify response structure
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    assert_eq!(body_json["status"], "healthy", "Status should be 'healthy'");
    assert!(body_json.get("version").is_some(), "Response should include version");
    assert_eq!(body_json["database"], "connected", "Database should be connected");
}

/// Test that /health/db endpoint includes database status
#[actix_web::test]
async fn test_health_db_includes_database_status() {
    let (service, _db) = lighter_auth::service!();

    let req = TestRequest::get()
        .uri("/health/db")
        .to_request();

    let resp = call_service(&service, req).await;
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: Value = serde_json::from_str(body_str).unwrap();

    // Verify response structure
    assert!(body_json.get("status").is_some(), "Response should include status");
    assert!(body_json.get("version").is_some(), "Response should include version");
    assert!(body_json.get("database").is_some(), "Response should include database");

    // Database should be a string ("connected" or "disconnected")
    assert!(body_json["database"].is_string(), "'database' should be a string");
    let db_status = body_json["database"].as_str().unwrap();
    assert!(db_status == "connected" || db_status == "disconnected", "database should be 'connected' or 'disconnected'");
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

/// Test that health endpoints respond quickly (performance check)
///
/// Health checks should be fast to avoid timeout issues in orchestrators.
/// This test ensures the endpoint responds within a reasonable time.
#[actix_web::test]
async fn test_health_endpoints_respond_quickly() {
    let (service, _db) = lighter_auth::service!();

    let start = std::time::Instant::now();

    let req = TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = call_service(&service, req).await;
    let duration = start.elapsed();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        duration.as_millis() < 100,
        "Health check should respond in less than 100ms, took {}ms",
        duration.as_millis()
    );
}

/// Test that readiness endpoint responds quickly even when checking dependencies
#[actix_web::test]
async fn test_ready_endpoint_responds_quickly() {
    let (service, _db) = lighter_auth::service!();

    let start = std::time::Instant::now();

    let req = TestRequest::get()
        .uri("/ready")
        .to_request();

    let resp = call_service(&service, req).await;
    let duration = start.elapsed();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        duration.as_millis() < 500,
        "Readiness check should respond in less than 500ms, took {}ms",
        duration.as_millis()
    );
}
