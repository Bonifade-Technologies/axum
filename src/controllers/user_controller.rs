use axum::{routing::get, Json, Router};

/// Returns a router containing all routes for the user controller.
pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/home", get(homepage))
}

async fn index() -> &'static str {
    "Hello, World!"
}

async fn homepage() -> Json<String> {
    Json("hello dear".to_string())
}
