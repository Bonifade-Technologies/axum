use axum::{routing::get, Json, Router};

/// Returns a router containing all routes for the user controller.
pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/show", get(show))
}

async fn index() -> &'static str {
    "User list"
}

async fn show() -> Json<String> {
    Json("user show".to_string())
}
