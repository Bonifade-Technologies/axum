use crate::controllers::{self};
use axum::{routing::get, Router};

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .nest("/users", controllers::user_controller::routes())
}

async fn index() -> &'static str {
    "Hello, World 2!"
}
