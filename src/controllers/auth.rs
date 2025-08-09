use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    Extension,
};
use chrono::Utc;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};

pub async fn register(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<SignupDto>,
) -> impl IntoResponse {
    // Double-check email uniqueness in database
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db)
        .await;

    if let Ok(Some(_)) = existing_user {
        return api_response::failure(
            Some("Registration failed"),
            Some("Email is already taken".to_string()),
            Some(StatusCode::BAD_REQUEST),
        );
    }

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
            // Store user in Redis with email as key
            let client = redis_client();
            if let Ok(mut conn) = client.get_async_connection().await {
                let resource = UserResource::from(&user);
                let user_json = serde_json::to_string(&resource).unwrap();

                // Set user data in Redis with 24-hour expiration
                let _: Result<(), redis::RedisError> = conn
                    .set_ex(format!("user:{}", user.email), user_json, 60 * 60 * 24)
                    .await;

                // Generate token
                let token = generate_token();
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
                api_response::success(
                    Some("User registered successfully, but session creation failed"),
                    Some(UserResource::from(&user)),
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
                // Generate token
                let token = generate_token();

                // Store token and user in Redis
                let client = redis_client();
                if let Ok(mut conn) = client.get_async_connection().await {
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

// Authentication middleware
pub async fn auth_middleware<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    // Get token from Authorization header
    let auth_header = request.headers().get("Authorization");
    match auth_header {
        Some(header) => {
            if let Ok(auth_str) = header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").trim();

                    // Verify token with Redis
                    let client = redis_client();
                    if let Ok(mut conn) = client.get_async_connection().await {
                        if let Ok(Some(user_email)) = conn
                            .get::<_, Option<String>>(format!("token:{}", token))
                            .await
                        {
                            // Get user from Redis or fall back to database
                            if let Ok(Some(user_json)) = conn
                                .get::<_, Option<String>>(format!("user:{}", user_email))
                                .await
                            {
                                if let Ok(user) = serde_json::from_str::<UserResource>(&user_json) {
                                    let req = request.with_extension(user);
                                    return next.run(req).await;
                                }
                            }

                            // If user not in Redis, get from database
                            let db = request.extensions().get::<DatabaseConnection>();
                            if let Some(db) = db {
                                let user_result = user::Entity::find()
                                    .filter(user::Column::Email.eq(user_email))
                                    .filter(user::Column::DeletedAt.is_null())
                                    .one(db)
                                    .await;

                                if let Ok(Some(user)) = user_result {
                                    let user_resource = UserResource::from(&user);
                                    let req = request.with_extension(user_resource);
                                    return next.run(req).await;
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }

    // Token is invalid or missing
    api_response::failure(
        Some("Unauthorized"),
        Some("Authentication required".to_string()),
        Some(StatusCode::UNAUTHORIZED),
    )
    .into_response()
}

// Profile function - protected by auth middleware
pub async fn profile(Extension(user): Extension<UserResource>) -> impl IntoResponse {
    api_response::success(Some("User profile"), Some(user), None)
}

// Helper functions
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_password(input: &str, stored: &str) -> bool {
    let hashed_input = hash_password(input);
    hashed_input == stored
}

fn generate_token() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
