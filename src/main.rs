#![deny(warnings)]

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

use std::io::Error;

use lighter_common::prelude::*;

#[actix::main]
async fn main() -> Result<(), Error> {
    tracing::init();

    let server = Server::env().await;

    server.run(router::route)?.await
}
