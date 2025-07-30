use crate::controllers::users::{create_user, delete_user, get_user, list_users, update_user};
use axum::{routing::get, Router};
use sea_orm::DatabaseConnection;

pub fn users_router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
}
