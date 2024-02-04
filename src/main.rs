#[macro_use]
extern crate actix_web;

pub mod api;
pub mod controllers;
pub mod entities;
pub mod middlewares;
pub mod models;
pub mod requests;
pub mod responses;
pub mod router;
pub mod services;

use std::env;
use std::io::Error;

use lighter_common::{prelude::*, tls};

#[actix::main]
async fn main() -> Result<(), Error> {
    tracing::init();

    let mut server = Server::env().await;

    server.tls(tls::configure(
        env::var("TLS_CERT").expect("TLS_CERT environment variable is required"),
        env::var("TLS_KEY").expect("TLS_KEY environment variable is required"),
    ));

    server.run(router::route)?.await
}
