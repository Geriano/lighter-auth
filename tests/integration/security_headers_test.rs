//! Integration tests for Security Headers Middleware
//!
//! These tests verify that security headers are properly added to all HTTP responses
//! across different routes, status codes, and configurations.

use actix_web::{test, web, App, HttpResponse};
use lighter_auth::config::SecurityHeadersConfig;
use lighter_auth::security::SecurityHeadersMiddleware;

/// Test that all required security headers are present with default configuration
#[actix_web::test]
async fn test_all_security_headers_present() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify all required headers are present
    assert!(
        resp.headers().contains_key("content-security-policy"),
        "Missing Content-Security-Policy header"
    );
    assert!(
        resp.headers().contains_key("strict-transport-security"),
        "Missing Strict-Transport-Security header"
    );
    assert!(
        resp.headers().contains_key("x-frame-options"),
        "Missing X-Frame-Options header"
    );
    assert!(
        resp.headers().contains_key("x-content-type-options"),
        "Missing X-Content-Type-Options header"
    );
    assert!(
        resp.headers().contains_key("referrer-policy"),
        "Missing Referrer-Policy header"
    );
    assert!(
        resp.headers().contains_key("x-xss-protection"),
        "Missing X-XSS-Protection header"
    );
    assert!(
        resp.headers()
            .contains_key("x-permitted-cross-domain-policies"),
        "Missing X-Permitted-Cross-Domain-Policies header"
    );
}

/// Test security header values match configuration defaults
#[actix_web::test]
async fn test_security_header_values_match_config() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config.clone()))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify header values
    assert_eq!(
        resp.headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap(),
        config.csp,
        "CSP header value mismatch"
    );

    assert_eq!(
        resp.headers()
            .get("strict-transport-security")
            .unwrap()
            .to_str()
            .unwrap(),
        format!("max-age={}; includeSubDomains", config.hsts_max_age),
        "HSTS header value mismatch"
    );

    assert_eq!(
        resp.headers()
            .get("x-frame-options")
            .unwrap()
            .to_str()
            .unwrap(),
        config.x_frame_options,
        "X-Frame-Options header value mismatch"
    );

    assert_eq!(
        resp.headers()
            .get("x-content-type-options")
            .unwrap()
            .to_str()
            .unwrap(),
        config.x_content_type_options,
        "X-Content-Type-Options header value mismatch"
    );

    assert_eq!(
        resp.headers()
            .get("referrer-policy")
            .unwrap()
            .to_str()
            .unwrap(),
        config.referrer_policy,
        "Referrer-Policy header value mismatch"
    );
}

/// Test security headers are added to success responses (200, 201, 204)
#[actix_web::test]
async fn test_security_headers_on_success_responses() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/ok", web::get().to(|| async { HttpResponse::Ok().body("OK") }))
            .route(
                "/created",
                web::post().to(|| async { HttpResponse::Created().body("Created") }),
            )
            .route(
                "/no-content",
                web::delete().to(|| async { HttpResponse::NoContent().finish() }),
            ),
    )
    .await;

    // Test 200 OK
    let req = test::TestRequest::get().uri("/ok").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().contains_key("content-security-policy"));
    assert!(resp.headers().contains_key("x-frame-options"));

    // Test 201 Created
    let req = test::TestRequest::post().uri("/created").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().contains_key("content-security-policy"));
    assert!(resp.headers().contains_key("x-frame-options"));

    // Test 204 No Content
    let req = test::TestRequest::delete().uri("/no-content").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.headers().contains_key("content-security-policy"));
    assert!(resp.headers().contains_key("x-frame-options"));
}

/// Test security headers are added to error responses (400, 401, 404, 500)
#[actix_web::test]
async fn test_security_headers_on_error_responses() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route(
                "/bad-request",
                web::get().to(|| async { HttpResponse::BadRequest().body("Bad Request") }),
            )
            .route(
                "/unauthorized",
                web::get().to(|| async { HttpResponse::Unauthorized().body("Unauthorized") }),
            )
            .route(
                "/not-found",
                web::get().to(|| async { HttpResponse::NotFound().body("Not Found") }),
            )
            .route(
                "/server-error",
                web::get().to(|| async {
                    HttpResponse::InternalServerError().body("Server Error")
                }),
            ),
    )
    .await;

    // Test 400 Bad Request
    let req = test::TestRequest::get().uri("/bad-request").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().contains_key("content-security-policy"),
        "CSP missing on 400 response"
    );
    assert!(
        resp.headers().contains_key("x-frame-options"),
        "X-Frame-Options missing on 400 response"
    );

    // Test 401 Unauthorized
    let req = test::TestRequest::get().uri("/unauthorized").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().contains_key("content-security-policy"),
        "CSP missing on 401 response"
    );

    // Test 404 Not Found
    let req = test::TestRequest::get().uri("/not-found").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().contains_key("content-security-policy"),
        "CSP missing on 404 response"
    );

    // Test 500 Internal Server Error
    let req = test::TestRequest::get().uri("/server-error").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().contains_key("content-security-policy"),
        "CSP missing on 500 response"
    );
}

/// Test security headers work with JSON responses
#[actix_web::test]
async fn test_security_headers_with_json_responses() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route(
                "/json",
                web::get().to(|| async {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                        "data": {
                            "message": "Hello, World!"
                        }
                    }))
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/json").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify security headers present with JSON
    assert!(resp.headers().contains_key("content-security-policy"));
    assert!(resp.headers().contains_key("strict-transport-security"));

    // Verify content-type is still JSON
    assert_eq!(
        resp.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "application/json"
    );
}

/// Test custom CSP configuration
#[actix_web::test]
async fn test_custom_csp_configuration() {
    let config = SecurityHeadersConfig {
        enabled: true,
        csp: "default-src 'self'; script-src 'self' https://cdn.example.com; style-src 'self' 'unsafe-inline'".to_string(),
        hsts_max_age: 31536000,
        x_frame_options: "DENY".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config.clone()))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap(),
        config.csp
    );
}

/// Test custom HSTS max-age configuration
#[actix_web::test]
async fn test_custom_hsts_max_age() {
    let config = SecurityHeadersConfig {
        enabled: true,
        csp: "default-src 'self'".to_string(),
        hsts_max_age: 63072000, // 2 years
        x_frame_options: "DENY".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.headers()
            .get("strict-transport-security")
            .unwrap()
            .to_str()
            .unwrap(),
        "max-age=63072000; includeSubDomains"
    );
}

/// Test custom X-Frame-Options configuration (SAMEORIGIN)
#[actix_web::test]
async fn test_custom_x_frame_options() {
    let config = SecurityHeadersConfig {
        enabled: true,
        csp: "default-src 'self'".to_string(),
        hsts_max_age: 31536000,
        x_frame_options: "SAMEORIGIN".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.headers()
            .get("x-frame-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "SAMEORIGIN"
    );
}

/// Test custom Referrer-Policy configuration
#[actix_web::test]
async fn test_custom_referrer_policy() {
    let config = SecurityHeadersConfig {
        enabled: true,
        csp: "default-src 'self'".to_string(),
        hsts_max_age: 31536000,
        x_frame_options: "DENY".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "no-referrer".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.headers()
            .get("referrer-policy")
            .unwrap()
            .to_str()
            .unwrap(),
        "no-referrer"
    );
}

/// Test that security headers are NOT added when disabled
#[actix_web::test]
async fn test_security_headers_disabled() {
    let config = SecurityHeadersConfig {
        enabled: false,
        csp: "default-src 'self'".to_string(),
        hsts_max_age: 31536000,
        x_frame_options: "DENY".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify NO security headers are added when disabled
    assert!(
        !resp.headers().contains_key("content-security-policy"),
        "CSP should not be present when disabled"
    );
    assert!(
        !resp.headers().contains_key("strict-transport-security"),
        "HSTS should not be present when disabled"
    );
    assert!(
        !resp.headers().contains_key("x-frame-options"),
        "X-Frame-Options should not be present when disabled"
    );
    assert!(
        !resp.headers().contains_key("x-content-type-options"),
        "X-Content-Type-Options should not be present when disabled"
    );
    assert!(
        !resp.headers().contains_key("referrer-policy"),
        "Referrer-Policy should not be present when disabled"
    );
}

/// Test security headers work with multiple routes
#[actix_web::test]
async fn test_security_headers_across_multiple_routes() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/route1", web::get().to(|| async { HttpResponse::Ok().body("Route 1") }))
            .route("/route2", web::post().to(|| async { HttpResponse::Ok().body("Route 2") }))
            .route("/route3", web::put().to(|| async { HttpResponse::Ok().body("Route 3") }))
            .route(
                "/route4",
                web::delete().to(|| async { HttpResponse::Ok().body("Route 4") }),
            ),
    )
    .await;

    let routes = vec![
        ("/route1", "GET"),
        ("/route2", "POST"),
        ("/route3", "PUT"),
        ("/route4", "DELETE"),
    ];

    for (path, method) in routes {
        let req = match method {
            "GET" => test::TestRequest::get().uri(path).to_request(),
            "POST" => test::TestRequest::post().uri(path).to_request(),
            "PUT" => test::TestRequest::put().uri(path).to_request(),
            "DELETE" => test::TestRequest::delete().uri(path).to_request(),
            _ => panic!("Unsupported method"),
        };

        let resp = test::call_service(&app, req).await;

        assert!(
            resp.headers().contains_key("content-security-policy"),
            "CSP missing on {} {}",
            method,
            path
        );
        assert!(
            resp.headers().contains_key("x-frame-options"),
            "X-Frame-Options missing on {} {}",
            method,
            path
        );
    }
}

/// Test security headers don't interfere with existing response headers
#[actix_web::test]
async fn test_security_headers_preserve_existing_headers() {
    let config = SecurityHeadersConfig::default();
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route(
                "/test",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .insert_header(("X-Custom-Header", "CustomValue"))
                        .insert_header(("X-Another-Header", "AnotherValue"))
                        .body("OK")
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify custom headers are still present
    assert_eq!(
        resp.headers()
            .get("x-custom-header")
            .unwrap()
            .to_str()
            .unwrap(),
        "CustomValue"
    );
    assert_eq!(
        resp.headers()
            .get("x-another-header")
            .unwrap()
            .to_str()
            .unwrap(),
        "AnotherValue"
    );

    // Verify security headers are also present
    assert!(resp.headers().contains_key("content-security-policy"));
    assert!(resp.headers().contains_key("x-frame-options"));
}

/// Test empty CSP is not added
#[actix_web::test]
async fn test_empty_csp_not_added() {
    let config = SecurityHeadersConfig {
        enabled: true,
        csp: "".to_string(), // Empty CSP
        hsts_max_age: 31536000,
        x_frame_options: "DENY".to_string(),
        x_content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
    };

    let app = test::init_service(
        App::new()
            .wrap(SecurityHeadersMiddleware::new(config))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().body("OK") })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify CSP is NOT added when empty
    assert!(!resp.headers().contains_key("content-security-policy"));

    // But other headers should still be present
    assert!(resp.headers().contains_key("strict-transport-security"));
    assert!(resp.headers().contains_key("x-frame-options"));
}
