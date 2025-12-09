//! # Security Headers Middleware
//!
//! Comprehensive security headers middleware for actix-web applications.
//! Adds essential HTTP security headers to all responses to protect against
//! common web vulnerabilities.
//!
//! ## Features
//!
//! - **Content Security Policy (CSP)**: Prevents XSS and other injection attacks
//! - **HTTP Strict Transport Security (HSTS)**: Forces HTTPS connections
//! - **X-Frame-Options**: Prevents clickjacking attacks
//! - **X-Content-Type-Options**: Prevents MIME type sniffing
//! - **Referrer-Policy**: Controls referrer information
//! - **X-XSS-Protection**: Legacy XSS protection (for older browsers)
//! - **X-Permitted-Cross-Domain-Policies**: Controls cross-domain policies
//!
//! ## Example
//!
//! ```rust,no_run
//! use lighter_auth::config::SecurityHeadersConfig;
//! use lighter_auth::security::SecurityHeadersMiddleware;
//! use actix_web::{App, HttpServer};
//!
//! #[actix_web::main]
//! async fn main() {
//!     let config = SecurityHeadersConfig::default();
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .wrap(SecurityHeadersMiddleware::new(config.clone()))
//!             // ... your routes
//!     })
//!     .bind("0.0.0.0:8080")
//!     .unwrap()
//!     .run()
//!     .await
//!     .unwrap()
//! }
//! ```

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use crate::config::SecurityHeadersConfig;

/// Security headers middleware for actix-web
///
/// This middleware adds comprehensive security headers to all HTTP responses
/// to protect against common web vulnerabilities including XSS, clickjacking,
/// MIME type sniffing, and more.
///
/// # Thread Safety
///
/// This middleware is thread-safe and can be cloned across worker threads.
///
/// # Configuration
///
/// The middleware is configurable via `SecurityHeadersConfig`, allowing you to:
/// - Enable/disable security headers globally
/// - Customize CSP policy
/// - Configure HSTS max-age
/// - Set X-Frame-Options value
/// - Customize other security header values
pub struct SecurityHeadersMiddleware {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersMiddleware {
    /// Create a new security headers middleware with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Security headers configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use lighter_auth::config::SecurityHeadersConfig;
    /// use lighter_auth::security::SecurityHeadersMiddleware;
    ///
    /// let config = SecurityHeadersConfig::default();
    /// let middleware = SecurityHeadersMiddleware::new(config);
    /// ```
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    /// Create middleware with default configuration
    ///
    /// This is a convenience method equivalent to:
    /// ```rust
    /// # use lighter_auth::config::SecurityHeadersConfig;
    /// # use lighter_auth::security::SecurityHeadersMiddleware;
    /// SecurityHeadersMiddleware::new(SecurityHeadersConfig::default())
    /// # ;
    /// ```
    pub fn default_config() -> Self {
        Self {
            config: SecurityHeadersConfig::default(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService {
            service,
            config: self.config.clone(),
        }))
    }
}

/// Security headers middleware service
///
/// This service adds security headers to all responses passing through it.
pub struct SecurityHeadersMiddlewareService<S> {
    service: S,
    config: SecurityHeadersConfig,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = self.config.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Only add headers if enabled
            if !config.enabled {
                return Ok(res);
            }

            // Get mutable reference to response headers
            let headers = res.headers_mut();

            // Content-Security-Policy
            // Prevents XSS, injection attacks, and unauthorized resource loading
            if !config.csp.is_empty() {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("content-security-policy"),
                    actix_web::http::header::HeaderValue::from_str(&config.csp)
                        .unwrap_or_else(|_| {
                            actix_web::http::header::HeaderValue::from_static("default-src 'self'")
                        }),
                );
            }

            // Strict-Transport-Security (HSTS)
            // Forces HTTPS connections for the specified duration
            let hsts_value = format!(
                "max-age={}; includeSubDomains",
                config.hsts_max_age
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::header::HeaderValue::from_str(&hsts_value)
                    .unwrap_or_else(|_| {
                        actix_web::http::header::HeaderValue::from_static(
                            "max-age=31536000; includeSubDomains",
                        )
                    }),
            );

            // X-Frame-Options
            // Prevents clickjacking attacks by controlling iframe embedding
            if !config.x_frame_options.is_empty() {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("x-frame-options"),
                    actix_web::http::header::HeaderValue::from_str(&config.x_frame_options)
                        .unwrap_or_else(|_| {
                            actix_web::http::header::HeaderValue::from_static("DENY")
                        }),
                );
            }

            // X-Content-Type-Options
            // Prevents MIME type sniffing
            if !config.x_content_type_options.is_empty() {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                    actix_web::http::header::HeaderValue::from_str(&config.x_content_type_options)
                        .unwrap_or_else(|_| {
                            actix_web::http::header::HeaderValue::from_static("nosniff")
                        }),
                );
            }

            // Referrer-Policy
            // Controls how much referrer information is included with requests
            if !config.referrer_policy.is_empty() {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("referrer-policy"),
                    actix_web::http::header::HeaderValue::from_str(&config.referrer_policy)
                        .unwrap_or_else(|_| {
                            actix_web::http::header::HeaderValue::from_static(
                                "strict-origin-when-cross-origin",
                            )
                        }),
                );
            }

            // X-XSS-Protection
            // Legacy XSS protection for older browsers (modern browsers use CSP)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // X-Permitted-Cross-Domain-Policies
            // Prevents Adobe Flash and PDF files from loading data from the domain
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "x-permitted-cross-domain-policies",
                ),
                actix_web::http::header::HeaderValue::from_static("none"),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_security_headers_middleware_default_config() {
        let config = SecurityHeadersConfig::default();
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify all security headers are present
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("strict-transport-security"));
        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(resp.headers().contains_key("x-content-type-options"));
        assert!(resp.headers().contains_key("referrer-policy"));
        assert!(resp.headers().contains_key("x-xss-protection"));
        assert!(resp
            .headers()
            .contains_key("x-permitted-cross-domain-policies"));
    }

    #[actix_web::test]
    async fn test_security_headers_values() {
        let config = SecurityHeadersConfig::default();
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
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
            "default-src 'self'"
        );
        assert_eq!(
            resp.headers()
                .get("strict-transport-security")
                .unwrap()
                .to_str()
                .unwrap(),
            "max-age=31536000; includeSubDomains"
        );
        assert_eq!(
            resp.headers()
                .get("x-frame-options")
                .unwrap()
                .to_str()
                .unwrap(),
            "DENY"
        );
        assert_eq!(
            resp.headers()
                .get("x-content-type-options")
                .unwrap()
                .to_str()
                .unwrap(),
            "nosniff"
        );
        assert_eq!(
            resp.headers()
                .get("referrer-policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            resp.headers()
                .get("x-xss-protection")
                .unwrap()
                .to_str()
                .unwrap(),
            "1; mode=block"
        );
        assert_eq!(
            resp.headers()
                .get("x-permitted-cross-domain-policies")
                .unwrap()
                .to_str()
                .unwrap(),
            "none"
        );
    }

    #[actix_web::test]
    async fn test_security_headers_custom_config() {
        let config = SecurityHeadersConfig {
            enabled: true,
            csp: "default-src 'self'; script-src 'self' https://cdn.example.com".to_string(),
            hsts_max_age: 63072000, // 2 years
            x_frame_options: "SAMEORIGIN".to_string(),
            x_content_type_options: "nosniff".to_string(),
            referrer_policy: "no-referrer".to_string(),
        };

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify custom header values
        assert_eq!(
            resp.headers()
                .get("content-security-policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "default-src 'self'; script-src 'self' https://cdn.example.com"
        );
        assert_eq!(
            resp.headers()
                .get("strict-transport-security")
                .unwrap()
                .to_str()
                .unwrap(),
            "max-age=63072000; includeSubDomains"
        );
        assert_eq!(
            resp.headers()
                .get("x-frame-options")
                .unwrap()
                .to_str()
                .unwrap(),
            "SAMEORIGIN"
        );
        assert_eq!(
            resp.headers()
                .get("referrer-policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "no-referrer"
        );
    }

    #[actix_web::test]
    async fn test_security_headers_disabled() {
        let config = SecurityHeadersConfig {
            enabled: false,
            ..SecurityHeadersConfig::default()
        };

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify no security headers are added when disabled
        assert!(!resp.headers().contains_key("content-security-policy"));
        assert!(!resp.headers().contains_key("strict-transport-security"));
        assert!(!resp.headers().contains_key("x-frame-options"));
        assert!(!resp.headers().contains_key("x-content-type-options"));
        assert!(!resp.headers().contains_key("referrer-policy"));
        assert!(!resp.headers().contains_key("x-xss-protection"));
        assert!(!resp
            .headers()
            .contains_key("x-permitted-cross-domain-policies"));
    }

    #[actix_web::test]
    async fn test_security_headers_on_error_responses() {
        let config = SecurityHeadersConfig::default();
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route(
                    "/error",
                    web::get().to(|| async { HttpResponse::InternalServerError().body("error") }),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/error").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify headers are added even on error responses
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("strict-transport-security"));
    }

    #[actix_web::test]
    async fn test_security_headers_on_different_status_codes() {
        let config = SecurityHeadersConfig::default();
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route(
                    "/ok",
                    web::get().to(|| async { HttpResponse::Ok().body("ok") }),
                )
                .route(
                    "/created",
                    web::post().to(|| async { HttpResponse::Created().body("created") }),
                )
                .route(
                    "/not-found",
                    web::get().to(|| async { HttpResponse::NotFound().body("not found") }),
                ),
        )
        .await;

        // Test 200 OK
        let req = test::TestRequest::get().uri("/ok").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.headers().contains_key("content-security-policy"));

        // Test 201 Created
        let req = test::TestRequest::post().uri("/created").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.headers().contains_key("content-security-policy"));

        // Test 404 Not Found
        let req = test::TestRequest::get().uri("/not-found").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.headers().contains_key("content-security-policy"));
    }

    #[actix_web::test]
    async fn test_security_headers_with_json_response() {
        let config = SecurityHeadersConfig::default();
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route(
                    "/json",
                    web::get().to(|| async {
                        HttpResponse::Ok().json(serde_json::json!({
                            "message": "test"
                        }))
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/json").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify headers are present with JSON responses
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("strict-transport-security"));
    }

    #[actix_web::test]
    async fn test_default_config_constructor() {
        let middleware = SecurityHeadersMiddleware::default_config();
        assert!(middleware.config.enabled);
        assert_eq!(middleware.config.csp, "default-src 'self'");
        assert_eq!(middleware.config.hsts_max_age, 31536000);
    }

    #[actix_web::test]
    async fn test_empty_csp_not_added() {
        let config = SecurityHeadersConfig {
            enabled: true,
            csp: "".to_string(), // Empty CSP
            ..SecurityHeadersConfig::default()
        };

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new(config))
                .route("/test", web::get().to(|| async { HttpResponse::Ok().body("test") })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify CSP header is not added when empty
        assert!(!resp.headers().contains_key("content-security-policy"));

        // But other headers should still be present
        assert!(resp.headers().contains_key("strict-transport-security"));
        assert!(resp.headers().contains_key("x-frame-options"));
    }
}
