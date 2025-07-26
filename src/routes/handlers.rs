use crate::{routes::samples::samples_router, utils::api_response};
use axum::{response::IntoResponse, routing::get, Router};

pub fn app_router() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
        .fallback(not_found)
}

async fn hello_world() -> impl IntoResponse {
    api_response::success(
        Some("Hello, world!"),
        Some("Welcome to axum apis. This is a template for building web applications with Rust and Axum."),
        None,
    )
}

async fn not_found() -> impl IntoResponse {
    api_response::failure(
        Some("Route not found"),
        Some("The requested endpoint does not exist."),
        None,
    )
}
