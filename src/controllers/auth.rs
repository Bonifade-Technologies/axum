use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::auth::{generate_jwt_token, hash_password, verify_password};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};
use chrono::Utc;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn register(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<SignupDto>,
) -> impl IntoResponse {
    let now = Utc::now().naive_utc();

    // Hash the password
    let hashed_password = hash_password(&payload.password);

    let user = user::ActiveModel {
        id: Set(cuid2::create_id()),
        name: Set(payload.name),
        email: Set(payload.email.clone()),
        phone: Set(Some(payload.phone)),
        password: Set(hashed_password),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let res = user.insert(&db).await;

    match res {
        Ok(user) => {
            // Generate JWT token
            let token = match generate_jwt_token(&user.email) {
                Ok(t) => t,
                Err(_) => {
                    return api_response::failure(
                        Some("Registration successful but token generation failed"),
                        Some("Failed to generate authentication token".to_string()),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    );
                }
            };

            // Store user in Redis with email as key
            let client = redis_client();
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let resource = UserResource::from(&user);
                let user_json = serde_json::to_string(&resource).unwrap();

                // Set user data in Redis with 24-hour expiration
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("user:{}", user.email), user_json, 60 * 60 * 24)
                    .await;

                // Store JWT token in Redis
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("token:{}", token), user.email.clone(), 60 * 60 * 24)
                    .await;

                // Return user with token
                let response = serde_json::json!({
                    "user": resource,
                    "token": token
                });

                api_response::success(
                    Some("User registered successfully"),
                    Some(response),
                    Some(StatusCode::CREATED),
                )
            } else {
                // Redis failed but registration was successful
                let response = serde_json::json!({
                    "user": UserResource::from(&user),
                    "token": token
                });

                api_response::success(
                    Some("User registered successfully"),
                    Some(response),
                    Some(StatusCode::CREATED),
                )
            }
        }
        Err(e) => api_response::failure(
            Some("Registration failed"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> impl IntoResponse {
    // Find user in database
    let user_result = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db)
        .await;

    match user_result {
        Ok(Some(user)) => {
            // Verify password
            if verify_password(&payload.password, &user.password) {
                // Generate JWT token
                let token = match generate_jwt_token(&user.email) {
                    Ok(t) => t,
                    Err(_) => {
                        return api_response::failure(
                            Some("Login successful but token generation failed"),
                            Some("Failed to generate authentication token".to_string()),
                            Some(StatusCode::INTERNAL_SERVER_ERROR),
                        );
                    }
                };

                // Store token and user in Redis
                let client = redis_client();
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let resource = UserResource::from(&user);
                    let user_json = serde_json::to_string(&resource).unwrap();

                    // Set token with 24-hour expiration
                    let _: Result<(), redis::RedisError> = conn
                        .set_ex(format!("token:{}", token), user.email.clone(), 60 * 60 * 24)
                        .await;

                    // Update user in Redis
                    let _: Result<(), redis::RedisError> = conn
                        .set_ex(format!("user:{}", user.email), user_json, 60 * 60 * 24)
                        .await;

                    // Return user with token
                    let response = serde_json::json!({
                        "user": resource,
                        "token": token
                    });

                    return api_response::success(Some("Login successful"), Some(response), None);
                }

                // Redis failed but login was successful
                let response = serde_json::json!({
                    "user": UserResource::from(&user),
                    "token": token
                });

                api_response::success(
                    Some("Login successful, but session creation failed"),
                    Some(response),
                    None,
                )
            } else {
                api_response::failure(
                    Some("Login failed"),
                    Some("Invalid password".to_string()),
                    Some(StatusCode::UNAUTHORIZED),
                )
            }
        }
        Ok(None) => api_response::failure(
            Some("Login failed"),
            Some("User not found".to_string()),
            Some(StatusCode::NOT_FOUND),
        ),
        Err(e) => api_response::failure(
            Some("Login failed"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

// Profile function - protected by auth middleware
pub async fn profile(Extension(user): Extension<UserResource>) -> impl IntoResponse {
    api_response::success(Some("User profile"), Some(user), None)
}
