use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
#[derive(Serialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

pub fn pagination_info(page: u64, per_page: u64, total: u64) -> Pagination {
    let total_pages = (total as f64 / per_page as f64).ceil() as u64;
    Pagination {
        page,
        per_page,
        total,
        total_pages,
    }
}

#[derive(Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct FailureResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
}

pub fn success<T: Serialize>(
    message: Option<&str>,
    data: Option<T>,
    status: Option<StatusCode>,
) -> Response {
    let resp = SuccessResponse {
        success: true,
        message: message
            .unwrap_or("Action completed successfully")
            .to_string(),
        data,
    };
    let status = status.unwrap_or(StatusCode::OK);
    (status, axum::Json(resp)).into_response()
}

pub fn failure<T: Serialize>(
    message: Option<&str>,
    errors: Option<T>,
    status: Option<StatusCode>,
) -> Response {
    let resp = FailureResponse {
        success: false,
        message: message.unwrap_or("An error occurred").to_string(),
        errors: errors.map(|e| serde_json::to_value(e).unwrap()),
    };
    let status = status.unwrap_or(StatusCode::BAD_REQUEST);
    (status, axum::Json(resp)).into_response()
}
