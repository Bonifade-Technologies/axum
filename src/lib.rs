use crate::utils::api_response;
use axum::{http::StatusCode, routing::get, Router};

pub mod config;
pub mod controllers;
pub mod models;
pub mod routes;
pub mod utils;
pub mod views;

pub async fn run() {
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world));

    // Use APP_URL and APP_PORT static variables from config/database.rs
    let addr = format!(
        "{}:{}",
        *crate::config::database::APP_URL,
        *crate::config::database::APP_PORT
    );

    println!("Starting server at {}", &addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Hello, world!"),
        Some("Welcome to axum apis. This is a template for building web applications with Rust and Axum."),
        Some(StatusCode::ACCEPTED),
    )
}
