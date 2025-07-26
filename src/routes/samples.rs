use crate::utils::api_response;
use axum::{http::StatusCode, routing::get, Router};

pub fn samples_router() -> Router {
    Router::new()
        .route("/ok", get(ok_handler))
        .route("/created", get(created_handler))
        .route("/accepted", get(accepted_handler))
        .route("/no_content", get(no_content_handler))
        .route("/bad_request", get(bad_request_handler))
        .route("/unauthorized", get(unauthorized_handler))
        .route("/forbidden", get(forbidden_handler))
        .route("/not_found", get(not_found_handler))
        .route("/conflict", get(conflict_handler))
        .route("/unprocessable_entity", get(unprocessable_entity_handler))
        .route("/too_many_requests", get(too_many_requests_handler))
        .route("/internal_server_error", get(internal_server_error_handler))
        .route("/bad_gateway", get(bad_gateway_handler))
        .route("/service_unavailable", get(service_unavailable_handler))
}

async fn ok_handler() -> impl axum::response::IntoResponse {
    api_response::success(Some("OK"), Some("Request succeeded"), Some(StatusCode::OK))
}

async fn created_handler() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Created"),
        Some("Resource created"),
        Some(StatusCode::CREATED),
    )
}

async fn accepted_handler() -> impl axum::response::IntoResponse {
    api_response::success(
        Some("Accepted"),
        Some("Request accepted"),
        Some(StatusCode::ACCEPTED),
    )
}

async fn no_content_handler() -> impl axum::response::IntoResponse {
    api_response::success(Some("No Content"), None::<()>, Some(StatusCode::NO_CONTENT))
}

async fn bad_request_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Bad Request"),
        Some("Invalid input"),
        Some(StatusCode::BAD_REQUEST),
    )
}

async fn unauthorized_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Unauthorized"),
        Some("Authentication required"),
        Some(StatusCode::UNAUTHORIZED),
    )
}

async fn forbidden_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Forbidden"),
        Some("Not allowed"),
        Some(StatusCode::FORBIDDEN),
    )
}

async fn not_found_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Not Found"),
        Some("Resource not found"),
        Some(StatusCode::NOT_FOUND),
    )
}

async fn conflict_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Conflict"),
        Some("Conflict with current state"),
        Some(StatusCode::CONFLICT),
    )
}

async fn unprocessable_entity_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Unprocessable Entity"),
        Some("Validation error"),
        Some(StatusCode::UNPROCESSABLE_ENTITY),
    )
}

async fn too_many_requests_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Too Many Requests"),
        Some("Rate limit exceeded"),
        Some(StatusCode::TOO_MANY_REQUESTS),
    )
}

async fn internal_server_error_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Internal Server Error"),
        Some("Server error"),
        Some(StatusCode::INTERNAL_SERVER_ERROR),
    )
}

async fn bad_gateway_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Bad Gateway"),
        Some("Invalid response from upstream"),
        Some(StatusCode::BAD_GATEWAY),
    )
}

async fn service_unavailable_handler() -> impl axum::response::IntoResponse {
    api_response::failure(
        Some("Service Unavailable"),
        Some("Service temporarily unavailable"),
        Some(StatusCode::SERVICE_UNAVAILABLE),
    )
}
