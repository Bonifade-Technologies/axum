use crate::controllers::admin_controller::{clear_all_caches, clear_user_cache};
use axum::{extract::Path, routing::delete, Router};

pub fn admin_routes() -> Router {
    Router::new()
        .route("/clear-cache", delete(clear_all_caches))
        .route("/clear-cache/:email", delete(clear_user_cache))
}
