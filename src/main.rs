#![deny(warnings)]

#[macro_use]
extern crate actix_web;

pub mod api;
pub mod cache;
pub mod config;
pub mod controllers;
pub mod entities;
pub mod middlewares;
pub mod models;
pub mod requests;
pub mod responses;
pub mod router;
pub mod services;

pub mod testing;

use std::io::Error;

use lighter_common::prelude::*;

#[actix::main]
async fn main() -> Result<(), Error> {
    // Load and validate configuration
    let app_config = config::load().expect("Failed to load configuration");

    // Initialize tracing
    tracing::init();

    // Initialize database with config
    let database = database::from_config(&app_config.database)
        .await
        .expect("Failed to connect to database");

    // Initialize server with config
    let server = Server::from_config(app_config.server, database);

    // Run server
    server.run(router::route)?.await
}
