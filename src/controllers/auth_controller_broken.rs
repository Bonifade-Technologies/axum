use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::auth::{
    generate_jwt_token, hash_password, unique_email, verify_password, 
    get_user_from_cache_or_db, cache_user_data, increment_user_activity,
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
            Some(StatusCode::UNPROCESSABLE_ENTITY),
        );
    }

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
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> impl IntoResponse {
    // Try to get user from cache first, fallback to database
    if let Some(cached_user) = get_user_from_cache_or_db(&payload.email).await {
        // Get fresh user data from DB for password verification
        let user_result = user::Entity::find()
            .filter(user::Column::Email.eq(&payload.email))
            .filter(user::Column::DeletedAt.is_null())
            .one(&db)
            .await;

        if let Ok(Some(user_model)) = user_result {
            // Verify password
            if verify_password(&payload.password, &user_model.password) {
                // Increment user activity for smart TTL
                increment_user_activity(&payload.email).await;
                
                // Generate JWT token
                let token = match generate_jwt_token(&user_model.email) {
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

                // Store token in Redis with session TTL
                let client = redis_client();
                if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
                    let _: Result<(), redis::RedisError> = conn
                        .set_ex(format!("token:{}", token), user_model.email.clone(), 24 * 60 * 60)
                        .await;
                    
                    // Update user cache with fresh data (extends TTL)
                    cache_user_data(&payload.email, &cached_user).await;
                }

                // Return user with token
                let response = serde_json::json!({
                    "user": cached_user,
                    "token": token
                });

                return api_response::success(Some("Login successful"), Some(response), None);
            } else {
                let error_response = serde_json::json!({
                    "password": "Invalid password"
                });
                return api_response::failure(
                    Some("Login failed"),
                    Some(error_response),
                    Some(StatusCode::UNAUTHORIZED),
                );
            }
        }
    }

    // Fallback: User not found in cache or database
    let error_response = serde_json::json!({
        "email": "User not found"
    });
    api_response::failure(
        Some("Login failed"),
        Some(error_response),
        Some(StatusCode::NOT_FOUND),
    )

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
        Ok(None) => {
            let error_response = serde_json::json!({
                "email": "User not found"
            });
            api_response::failure(
                Some("Login failed"),
                Some(error_response),
                Some(StatusCode::NOT_FOUND),
            )
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "database": e.to_string()
            });
            api_response::failure(
                Some("Login failed"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}

// Profile function - protected by auth middleware
pub async fn profile(Extension(user): Extension<UserResource>) -> impl IntoResponse {
    api_response::success(Some("User profile"), Some(user), None)
}
