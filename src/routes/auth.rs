use crate::controllers::auth_controller::{login, logout, profile, register, forgot_password, reset_password};
use crate::middlewares::auth_middleware::auth_middleware;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;

pub fn auth_router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route(
            "/profile",
            get(profile).route_layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/logout",
            post(logout).route_layer(middleware::from_fn(auth_middleware)),
        )
}
