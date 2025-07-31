#[macro_export]
macro_rules! validate_payload {
    ($payload:expr) => {
        match $payload.validate() {
            Ok(_) => Ok($payload),
            Err(errors) => {
                let error_map = validation_errors_to_map(&errors);
                return api_response::failure(
                    Some("Validation error"),
                    Some(error_map),
                    Some(StatusCode::UNPROCESSABLE_ENTITY),
                );
            }
        }
    };
}
