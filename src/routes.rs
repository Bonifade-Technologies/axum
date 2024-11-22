use crate::controllers::{self};
use crate::views::response::ApiResponse;
use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/errors/:code", get(simulate_error))
        .nest("/users", controllers::user_controller::routes())
}

async fn index() -> &'static str {
    "Hello, World 2!"
}

// Example route: Simulate an error
async fn simulate_error(Path(code): Path<u16>) -> (StatusCode, Json<ApiResponse>) {
    match code {
        400 => ApiResponse::failure("Bad request", Some(StatusCode::BAD_REQUEST)),
        500 => ApiResponse::failure(
            "Internal server error",
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
        503 => ApiResponse::failure("Service unavailable", Some(StatusCode::SERVICE_UNAVAILABLE)),
        501 => ApiResponse::failure("Not implemented", Some(StatusCode::NOT_IMPLEMENTED)),
        401 => ApiResponse::failure("Unauthorized", Some(StatusCode::UNAUTHORIZED)),
        403 => ApiResponse::failure("Forbidden", Some(StatusCode::FORBIDDEN)),
        _ => ApiResponse::failure("Unknown error", Some(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}
