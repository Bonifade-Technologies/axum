use crate::utils::api_response;
use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
// no need for Future import with async fn
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
            let payload = match Json::<T>::from_request(req, state).await {
                Ok(Json(payload)) => payload,
                Err(rejection) => {
                    let error_message = match rejection {
                        JsonRejection::JsonDataError(_err) => {
                            format!("Kindly pass a valid JSON data: {_err}")
                        }
                        JsonRejection::JsonSyntaxError(_err) => {
                            "Kindly pass a valid JSON body".to_string()
                        }
                        JsonRejection::MissingJsonContentType(_) => {
                            "Content-Type must be application/json".to_string()
                        }
                        _ => "Failed to parse request".to_string(),
                    };

                    return Err(api_response::failure(
                        Some("Invalid request"),
                        Some(error_message),
                        Some(StatusCode::BAD_REQUEST),
                    ));
                }
            };

            if let Err(validation_errors) = payload.validate() {
                let mut error_map = HashMap::new();

                for (field, errors) in validation_errors.field_errors() {
                    if let Some(error) = errors.first() {
                        let message = error
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("{field} is invalid"));
                        error_map.insert(field.to_string(), message);
                    }
                }

                return Err(api_response::failure(
                    Some("Validation failed"),
                    Some(error_map),
                    Some(StatusCode::UNPROCESSABLE_ENTITY),
                ));
            }

            Ok(ValidatedJson(payload))
        }
    }
