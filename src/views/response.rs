use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

impl ApiResponse {
    // Success response
    pub fn success(
        message: &str,
        data: Option<impl Serialize>,
        status: Option<StatusCode>,
    ) -> (StatusCode, Json<ApiResponse>) {
        let serialized_data = data.map(|d| serde_json::to_value(d).unwrap());
        let status_code = status.unwrap_or(StatusCode::OK); // Use provided status or default to OK
        (
            status_code,
            Json(ApiResponse {
                success: true,
                message: message.to_string(),
                data: serialized_data,
            }),
        )
    }

    // Failure response
    pub fn failure(message: &str, status: Option<StatusCode>) -> (StatusCode, Json<ApiResponse>) {
        let status_code = status.unwrap_or(StatusCode::BAD_REQUEST); // Use provided status or default to BAD_REQUEST
        (
            status_code,
            Json(ApiResponse {
                success: false,
                message: message.to_string(),
                data: None,
            }),
        )
    }
}
