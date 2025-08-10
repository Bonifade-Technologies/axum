mod admin_route;
mod auth_route;
mod samples;
mod users;

use crate::{
    routes::admin_route::admin_routes, routes::auth_route::auth_router,
    routes::samples::samples_router, routes::users::users_router, utils::api_response,
};

use axum::{http::StatusCode, routing::get, Router};
use sea_orm::DatabaseConnection;

pub fn app_router(database: DatabaseConnection) -> Router {
    Router::new()
        .route("/", get(hello_world))
        .nest("/samples", samples_router())
        .nest("/auth", auth_router().with_state(database.clone()))
        .nest("/users", users_router().with_state(database.clone()))
        .nest("/admin", admin_routes())
}

async fn hello_world() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Hello, world!"),
        Some("Welcome to axum apis. This is a template for building web applications with Rust and Axum."),
        Some(StatusCode::ACCEPTED),
    )
}
