use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::auth::{generate_jwt_token, hash_password, unique_email, verify_password};

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
        name: Set(payload.name.clone()),
        email: Set(payload.email.clone()),
        phone: Set(Some(payload.phone.clone())),
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
                let user_json = serde_json::to_string(&resource).unwrap();

                // Set user data in Redis with 7-day sliding window
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("user:{}", user.email), user_json, 7 * 24 * 60 * 60)
                    .await;

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
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> impl IntoResponse {
    let client = redis_client();
    let mut use_cached_user = false;
    let mut cached_user_resource: Option<UserResource> = None;

    // Try to get user from Redis cache first
    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{}", payload.email);
        let cached_user: Result<String, redis::RedisError> = conn.get(&redis_key).await;

        if let Ok(user_json) = cached_user {
            if let Ok(user_resource) = serde_json::from_str::<UserResource>(&user_json) {
                // Extend TTL on cache hit (sliding window)
                let _: Result<(), redis::RedisError> =
                    conn.expire(&redis_key, 7 * 24 * 60 * 60).await;

                // Increment activity counter
                let activity_key = format!("activity:{}", payload.email);
                let _: Result<i64, redis::RedisError> = conn.incr(&activity_key, 1).await;
                let _: Result<(), redis::RedisError> =
                    conn.expire(&activity_key, 30 * 24 * 60 * 60).await;

                cached_user_resource = Some(user_resource);
                use_cached_user = true;
                println!("✅ Cache HIT for user: {}", payload.email);
            }
        }
    }

    // Always fetch user from database for password verification
    let user_result = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db)
        .await;

    match user_result {
        Ok(Some(user_model)) => {
            // Verify password
            if verify_password(&payload.password, &user_model.password) {
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

                // Store token and user in Redis
                if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
                    // Store token with 24-hour expiration
                    let _: Result<(), redis::RedisError> = conn
                        .set_ex(
                            format!("token:{}", token),
                            user_model.email.clone(),
                            24 * 60 * 60,
                        )
                        .await;

                    // Update/store user in Redis if not cached or update cache
                    if !use_cached_user {
                        let resource = UserResource::from(&user_model);
                        let user_json = serde_json::to_string(&resource).unwrap();
                        let _: Result<(), redis::RedisError> = conn
                            .set_ex(
                                format!("user:{}", user_model.email),
                                user_json,
                                7 * 24 * 60 * 60,
                            )
                            .await;
                        cached_user_resource = Some(resource);
                        println!(
                            "💾 Cache MISS for user: {} - cached from database",
                            payload.email
                        );
                    }
                }

                // Return user with token
                let user_to_return =
                    cached_user_resource.unwrap_or_else(|| UserResource::from(&user_model));
                let response = serde_json::json!({
                    "user": user_to_return,
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
