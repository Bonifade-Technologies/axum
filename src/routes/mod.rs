mod samples;
mod users;

use crate::{routes::samples::samples_router, routes::users::users_router, utils::api_response};
use axum::{http::StatusCode, routing::get, Router};
use sea_orm::DatabaseConnection;

pub fn app_router(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
        .nest("/users", users_router(db))
}

async fn hello_world() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Hello, world!"),
        Some("Welcome to axum apis. This is a template for building web applications with Rust and Axum."),
        Some(StatusCode::ACCEPTED),
    )
}
