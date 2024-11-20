use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    // Success response
    pub fn success(message: &str, data: T) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::OK, // 200 OK
            Json(ApiResponse {
                success: true,
                message: message.to_string(),
                data: Some(data),
            }),
        )
    }

    // Failure response
    pub fn failure(message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            StatusCode::BAD_REQUEST, // 400 Bad Request
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }

    // Not Found response
    pub fn not_found(message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            StatusCode::NOT_FOUND, // 404 Not Found
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }

    // Internal Server Error
    pub fn internal_error(message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR, // 500 Internal Server Error
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }

    // Unauthorized response
    pub fn unauthorized(message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            StatusCode::UNAUTHORIZED, // 401 Unauthorized
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }

    // Forbidden response
    pub fn forbidden(message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
        (
            StatusCode::FORBIDDEN, // 403 Forbidden
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }
}
