use crate::controllers::auth_controller::{login, logout, profile, register};
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
        .route(
            "/profile",
            get(profile).route_layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/logout",
            post(logout).route_layer(middleware::from_fn(auth_middleware)),
        )
}
