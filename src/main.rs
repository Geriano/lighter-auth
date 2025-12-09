#![deny(warnings)]

#[macro_use]
extern crate actix_web;

pub mod api;
pub mod cache;
pub mod config;
pub mod controllers;
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
    web::{Data, FormConfig, JsonConfig, PathConfig, PayloadConfig},
    App, HttpServer,
};
use lighter_common::prelude::*;

use security::SecurityHeadersMiddleware;

#[actix::main]
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
    let database = database::from_config(&app_config.database)
        .await
        .expect("Failed to connect to database");

    ::tracing::info!(
        database_url = %app_config.database.url,
        "Database connection established"
    );

    // Extract config for server setup
    let addr: SocketAddr = format!("{}:{}", app_config.server.host, app_config.server.port)
        .parse()
        .expect("Failed to parse server address");

    let max_payload = app_config.server.max_payload_size;
    let workers = app_config.server.workers;
    let security_headers_config = app_config.security.headers.clone();

    ::tracing::info!(
        host = %app_config.server.host,
        port = %app_config.server.port,
        workers = %workers,
        security_headers_enabled = %security_headers_config.enabled,
        "Configuring HTTP server"
    );

    // Create server with custom middleware configuration
    let mut http_server = HttpServer::new(move || {
        let payload = PayloadConfig::new(max_payload);
        let path = PathConfig::default();
        let json = JsonConfig::default().limit(max_payload);
        let form = FormConfig::default().limit(max_payload);

        App::new()
            // Add security headers middleware first (applies to all routes)
            .wrap(SecurityHeadersMiddleware::new(security_headers_config.clone()))
            .app_data(payload)
            .app_data(path)
            .app_data(json)
            .app_data(form)
            .app_data(Data::new(database.clone()))
            .configure(router::route)
    });

    // Set workers (0 = auto-detect CPU count)
    if workers > 0 {
        http_server = http_server.workers(workers);
    }

    ::tracing::info!(
        address = %addr,
        "Server listening and ready to accept connections"
    );

    // Run server
    http_server.bind(addr)?.run().await
}
