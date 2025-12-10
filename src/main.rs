#![deny(warnings)]

#[macro_use]
extern crate actix_web;

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

pub mod testing;

use std::io::Error;
use std::net::SocketAddr;

use actix_web::{
    middleware::{Compress, Logger},
    web::{Data, FormConfig, JsonConfig, PathConfig, PayloadConfig},
    App, HttpServer,
};
use lighter_common::prelude::*;
use lighter_common::database as common_database;

use database::DatabasePool;
use security::{SecurityHeadersMiddleware, RateLimitMiddleware};
use middlewares::v1::auth::Authenticated;
use cache::{HybridCache, LocalCache, RedisCache};
use metrics::{AppMetrics, MetricsMiddleware};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    // Load and validate configuration
    let app_config = config::load().expect("Failed to load configuration");

    // Initialize tracing
    tracing::init();

    ::tracing::info!(
        app_name = %app_config.app.name,
        app_version = %app_config.app.version,
        environment = %app_config.app.environment,
        "Starting lighter-auth service"
    );

    // Initialize database with config
    let database_connection = common_database::from_config(&app_config.database)
        .await
        .expect("Failed to connect to database");

    ::tracing::info!(
        database_url = %app_config.database.url,
        "Database connection established"
    );

    // Wrap database connection with circuit breaker protection
    let database = DatabasePool::new(database_connection, app_config.resilience.clone());

    ::tracing::info!(
        circuit_breaker_enabled = %database.is_circuit_breaker_enabled(),
        "Database pool with circuit breaker initialized"
    );

    // Initialize cache (HybridCache with L1 + optional L2)
    let l1 = LocalCache::new();

    let l2 = match std::env::var("REDIS_URL") {
        Ok(url) => {
            ::tracing::info!(redis_url = %url, "Attempting to connect to Redis");
            match RedisCache::new(&url, "lighter-auth").await {
                Ok(redis) => {
                    ::tracing::info!("Redis connection established, using hybrid cache (L1 + L2)");
                    Some(redis)
                }
                Err(e) => {
                    ::tracing::warn!(error = %e, "Failed to connect to Redis, using local cache only");
                    None
                }
            }
        }
        Err(_) => {
            ::tracing::info!("REDIS_URL not set, using local cache only");
            None
        }
    };

    let cache_type = if l2.is_some() { "hybrid (L1+L2)" } else { "local (L1 only)" };
    let cache = std::sync::Arc::new(HybridCache::new(l1, l2));

    ::tracing::info!(
        cache_type = cache_type,
        "Cache initialized"
    );

    // Initialize metrics
    let metrics = AppMetrics::new();

    ::tracing::info!(
        "Metrics initialized with Prometheus exporter"
    );

    // Create authenticated middleware with cache
    let authenticated = Data::new(Authenticated::new(cache));

    // Extract database connection for handlers (clone the Arc)
    // We keep the DatabasePool for health checks but also provide the raw connection
    let db_connection = database.connection().clone();

    // Extract config for server setup
    let addr: SocketAddr = format!("{}:{}", app_config.server.host, app_config.server.port)
        .parse()
        .expect("Failed to parse server address");

    let max_payload = app_config.server.max_payload_size;
    let workers = app_config.server.workers;
    let shutdown_timeout = app_config.app.shutdown_timeout;
    let security_headers_config = app_config.security.headers.clone();
    let rate_limit_config = app_config.security.rate_limit.clone();

    ::tracing::info!(
        host = %app_config.server.host,
        port = %app_config.server.port,
        workers = %workers,
        shutdown_timeout = %shutdown_timeout,
        security_headers_enabled = %security_headers_config.enabled,
        rate_limit_enabled = %rate_limit_config.enabled,
        rate_limit_requests = %rate_limit_config.requests,
        rate_limit_window = %rate_limit_config.window,
        "Configuring HTTP server"
    );

    // Create server with custom middleware configuration
    let mut http_server = HttpServer::new(move || {
        let payload = PayloadConfig::new(max_payload);
        let path = PathConfig::default();
        let json = JsonConfig::default().limit(max_payload);
        let form = FormConfig::default().limit(max_payload);

        // Convert RateLimitConfig to the format expected by RateLimitMiddleware
        let rate_limit_middleware_config = crate::security::rate_limit::RateLimitConfig {
            requests_per_window: rate_limit_config.requests,
            window_seconds: rate_limit_config.window,
            burst_capacity: rate_limit_config.burst,
            enabled: rate_limit_config.enabled,
        };

        App::new()
            // Middleware order (outer to inner):
            // 1. MetricsMiddleware - tracks all requests (must be outermost)
            .wrap(MetricsMiddleware::new(metrics.clone()))
            // 2. Logger - logs all requests
            .wrap(Logger::default())
            // 3. Compress - compresses responses
            .wrap(Compress::default())
            // 4. RateLimitMiddleware - rate limits requests per IP
            .wrap(RateLimitMiddleware::new(rate_limit_middleware_config))
            // 5. SecurityHeadersMiddleware - applies security headers
            .wrap(SecurityHeadersMiddleware::new(security_headers_config.clone()))
            // App data (available to all handlers)
            .app_data(payload)
            .app_data(path)
            .app_data(json)
            .app_data(form)
            .app_data(Data::new(db_connection.clone()))  // For regular handlers
            .app_data(Data::new(database.clone()))       // For health checks
            .app_data(Data::clone(&authenticated))
            .app_data(Data::new(metrics.clone()))
            // Configure routes
            .configure(router::route)
    });

    // Set workers (0 = auto-detect CPU count)
    if workers > 0 {
        http_server = http_server.workers(workers);
    }

    // Configure graceful shutdown timeout
    http_server = http_server.shutdown_timeout(shutdown_timeout);

    ::tracing::info!(
        address = %addr,
        "Server listening and ready to accept connections"
    );

    // Bind and create server
    let server = http_server.bind(addr)?.run();

    // Get server handle for graceful shutdown
    let server_handle = server.handle();

    // Spawn shutdown signal handler
    tokio::spawn(async move {
        shutdown_signal().await;
        ::tracing::info!(
            shutdown_timeout_seconds = shutdown_timeout,
            "Received shutdown signal, initiating graceful shutdown and draining in-flight requests"
        );
        // Trigger graceful shutdown (graceful=true means wait for in-flight requests)
        server_handle.stop(true).await;
    });

    // Run server (this blocks until shutdown signal)
    let result = server.await;

    // Log shutdown completion
    match result {
        Ok(_) => {
            ::tracing::info!("Graceful shutdown completed successfully");
            Ok(())
        }
        Err(e) => {
            ::tracing::error!(error = %e, "Server shutdown with error");
            Err(e)
        }
    }
}

/// Wait for shutdown signal (SIGTERM or Ctrl+C)
///
/// This function listens for:
/// - SIGTERM: Typical termination signal from Docker/Kubernetes/systemd
/// - SIGINT: Ctrl+C in development/terminal
///
/// The function returns when either signal is received, triggering graceful shutdown.
#[allow(dead_code)]
async fn shutdown_signal() {
    use tokio::signal;

    // Handle Ctrl+C (SIGINT)
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        ::tracing::debug!("Received SIGINT (Ctrl+C)");
    };

    // Handle SIGTERM (Unix-only)
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
        ::tracing::debug!("Received SIGTERM");
    };

    // On non-Unix platforms (Windows), only handle Ctrl+C
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either signal
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
