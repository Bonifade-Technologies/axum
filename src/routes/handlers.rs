use crate::{routes::samples::samples_router, utils::api_response};
use axum::{http::StatusCode, routing::get, Router};

pub fn app_router() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
}

async fn hello_world() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Hello, world!"),
        Some("Welcome to axum apis. This is a template for building web applications with Rust and Axum."),
        Some(StatusCode::ACCEPTED),
    )
}
