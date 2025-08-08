use crate::controllers::users::{
    create_user, delete_user, force_delete_user, get_user, list_deleted_users, list_users,
    restore_user, update_user,
};
use axum::{
    routing::{delete, get, post},
    Router,
};
use sea_orm::DatabaseConnection;

pub fn users_router() -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/deleted", get(list_deleted_users))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/force-delete", delete(force_delete_user))
        .route("/{id}/restore", post(restore_user))
}
