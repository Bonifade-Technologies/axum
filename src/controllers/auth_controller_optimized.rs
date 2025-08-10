use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::auth::{
    generate_jwt_token, hash_password, unique_email,
    exist_email, authenticate_user, cache_complete_user_data, CachedUser,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};
use chrono::Utc;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn register(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<SignupDto>,
) -> impl IntoResponse {
    let now = Utc::now().naive_utc();

    let unique = unique_email(&payload.email).await;

    if !unique {
        let error_response = serde_json::json!({
            "email": "Email is already taken"
        });
        return api_response::failure(
            Some("User with email already exists"),
            Some(error_response),
            Some(StatusCode::CONFLICT),
        );
    }

    // Hash the password
    let hashed_password = hash_password(&payload.password);

    let user = user::ActiveModel {
        id: Set(cuid2::create_id()),
        name: Set("User".to_string()), // Default name, can be updated later
        email: Set(payload.email.clone()),
        phone: Set(None), // Optional field
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
                    let error_response = serde_json::json!({
                        "token": "Failed to generate authentication token"
                    });
                    return api_response::failure(
                        Some("Registration successful but token generation failed"),
                        Some(error_response),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    );
                }
            };

            // Store user in Redis with email as key
            let client = redis_client();
            if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
                let resource = UserResource::from(&user);
                
                // Cache complete user data including password hash
                let cached_user = CachedUser {
                    user_resource: resource.clone(),
                    password_hash: user.password.clone(),
                };
                cache_complete_user_data(&user.email, &cached_user).await;

                // Store JWT token in Redis
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("token:{}", token), user.email.clone(), 24 * 60 * 60)
                    .await;

                // Initialize activity counter
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("activity:{}", user.email), 1, 30 * 24 * 60 * 60)
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
        Err(e) => {
            let error_response = serde_json::json!({
                "database": e.to_string()
            });
            api_response::failure(
                Some("Registration failed"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}

pub async fn login(
    State(_db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> impl IntoResponse {
    // First check if user exists using our efficient exist_email function
    if !exist_email(&payload.email).await {
        let error_response = serde_json::json!({
            "email": "User not found"
        });
        return api_response::failure(
            Some("Login failed"),
            Some(error_response),
            Some(StatusCode::NOT_FOUND),
        );
    }

    // Use the smart authentication function (cache-first with password verification)
    if let Some(user_resource) = authenticate_user(&payload.email, &payload.password).await {
        // Generate JWT token
        let token = match generate_jwt_token(&payload.email) {
            Ok(t) => t,
            Err(_) => {
                let error_response = serde_json::json!({
                    "token": "Failed to generate authentication token"
                });
                return api_response::failure(
                    Some("Login successful but token generation failed"),
                    Some(error_response),
                    Some(StatusCode::INTERNAL_SERVER_ERROR),
                );
            }
        };

        // Store token in Redis
        let client = redis_client();
        if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
            let _: Result<(), redis::RedisError> = conn
                .set_ex(format!("token:{}", token), payload.email.clone(), 24 * 60 * 60)
                .await;
        }

        // Return user with token
        let response = serde_json::json!({
            "user": user_resource,
            "token": token
        });

        api_response::success(Some("Login successful"), Some(response), None)
    } else {
        let error_response = serde_json::json!({
            "password": "Invalid password"
        });
        api_response::failure(
            Some("Login failed"),
            Some(error_response),
            Some(StatusCode::UNAUTHORIZED),
        )
    }
}

// Profile function - protected by auth middleware
pub async fn profile(Extension(user): Extension<UserResource>) -> impl IntoResponse {
    api_response::success(Some("User profile"), Some(user), None)
}
