use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};

use crate::{models::user::User, views::response::ApiResponse};

/// Returns a router containing all routes for the user controller.
pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/show/:id", get(show))
}

async fn index() -> (StatusCode, Json<ApiResponse>) {
    ApiResponse::success("List of users", Some(()), Some(StatusCode::CREATED))
}

async fn show(Path(id): Path<u32>) -> (StatusCode, Json<ApiResponse>) {
    // Simulate a user found
    let user = User {
        id,
        name: "John Doe".to_string(),
    };
    // Return a success response
    ApiResponse::success("User found", Some(user), None)
}
