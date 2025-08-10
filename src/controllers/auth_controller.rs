use crate::config::redis::redis_client;
use crate::config::email::send_otp_email;
use crate::database::users as user;
use crate::dtos::auth_dto::{LoginDto, SignupDto, ForgotPasswordDto, ResetPasswordDto};
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::auth::{
    authenticate_user, cache_complete_user_data, exist_email, generate_jwt_token, hash_password,
    invalidate_all_user_tokens, unique_email, verify_password, CachedUser, generate_otp, 
    store_otp, verify_and_consume_otp, update_user_password, get_user_from_cache_or_db,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};
use chrono::Utc;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

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
            // Verify the password hash before generating token
            if !verify_password(&payload.password, &user.password) {
                let error_response = serde_json::json!({
                    "password": "Password verification failed after hashing"
                });
                return api_response::failure(
                    Some("Registration failed"),
                    Some(error_response),
                    Some(StatusCode::INTERNAL_SERVER_ERROR),
                );
            }

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
    if !exist_email(&payload.email).await {
        let error_response = serde_json::json!({
            "email": "User not found, kindly register"
        });
        return api_response::failure(
            Some("Login failed"),
            Some(error_response),
            Some(StatusCode::UNPROCESSABLE_ENTITY),
        );
    }

    // Use the smart authentication function (cache-first with password verification)
    if let Some(user_resource) = authenticate_user(&payload.email, &payload.password).await {
        // SECURITY: Invalidate all existing tokens for this user before creating a new one
        // This ensures only one active session per user (you can modify this behavior)
        match invalidate_all_user_tokens(&payload.email).await {
            Ok(count) => {
                if count > 0 {
                    println!(
                        "DEBUG: Invalidated {} existing tokens for {}",
                        count, payload.email
                    );
                }
            }
            Err(e) => {
                println!("DEBUG: Failed to invalidate old tokens: {}", e);
                // We continue anyway - this shouldn't block login
            }
        }

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
                .set_ex(
                    format!("token:{}", token),
                    payload.email.clone(),
                    24 * 60 * 60,
                )
                .await;
        }

        // Return user with token
        let response = serde_json::json!({
            "user": user_resource,
            "token": token
        });

        api_response::success(
            Some("Login successful"),
            Some(response),
            Some(StatusCode::OK),
        )
    } else {
        let error_response = serde_json::json!({
            "password": "incorrect password"
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

// Logout function - invalidates the current token
pub async fn logout(
    axum::extract::Extension(user): axum::extract::Extension<UserResource>,
) -> impl IntoResponse {
    // Since we have the user from middleware, we can invalidate all their tokens
    // This is actually more secure than just invalidating the current token

    let client = redis_client();
    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // Get all tokens for this user and delete them
            let token_pattern = "token:*";
            let all_token_keys: Result<Vec<String>, redis::RedisError> =
                conn.keys(token_pattern).await;

            let mut invalidated_count = 0;

            if let Ok(token_keys) = all_token_keys {
                for token_key in token_keys {
                    let stored_email: Result<String, redis::RedisError> =
                        conn.get(&token_key).await;

                    if let Ok(stored_email) = stored_email {
                        if stored_email == user.email {
                            let deleted: Result<i32, redis::RedisError> =
                                conn.del(&token_key).await;
                            if let Ok(count) = deleted {
                                invalidated_count += count;
                            }
                        }
                    }
                }
            }

            api_response::success(
                Some("Logout successful"),
                Some(serde_json::json!({
                    "message": "All sessions invalidated successfully",
                    "invalidated_tokens": invalidated_count
                })),
                Some(StatusCode::OK),
            )
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "redis": format!("Redis connection failed: {}", e)
            });
            api_response::failure(
                Some("Logout failed"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}

// Forgot password function - sends OTP to user's email
pub async fn forgot_password(
    ValidatedJson(payload): ValidatedJson<ForgotPasswordDto>,
) -> impl IntoResponse {
    // Check if user exists
    if !exist_email(&payload.email).await {
        let error_response = serde_json::json!({
            "email": "No account found with this email address"
        });
        return api_response::failure(
            Some("Email not found"),
            Some(error_response),
            Some(StatusCode::NOT_FOUND),
        );
    }

    // Get user data to get their name for personalized email
    let user_data = match get_user_from_cache_or_db(&payload.email).await {
        Some(user) => user,
        None => {
            let error_response = serde_json::json!({
                "system": "Unable to process password reset at this time"
            });
            return api_response::failure(
                Some("System error"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            );
        }
    };

    // Generate OTP
    let otp = generate_otp();

    // Store OTP in Redis (10 minutes expiration)
    match store_otp(&payload.email, &otp).await {
        Ok(_) => {
            // Send OTP email
            match send_otp_email(&payload.email, &user_data.name, &otp).await {
                Ok(_) => {
                    api_response::success(
                        Some("OTP sent successfully"),
                        Some(serde_json::json!({
                            "message": "Password reset OTP has been sent to your email",
                            "email": payload.email,
                            "expires_in_minutes": 10
                        })),
                        Some(StatusCode::OK),
                    )
                }
                Err(e) => {
                    println!("ERROR: Failed to send email: {}", e);
                    let error_response = serde_json::json!({
                        "email": "Failed to send reset email. Please try again later."
                    });
                    api_response::failure(
                        Some("Email sending failed"),
                        Some(error_response),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                }
            }
        }
        Err(e) => {
            println!("ERROR: Failed to store OTP: {}", e);
            let error_response = serde_json::json!({
                "system": "Unable to process password reset request"
            });
            api_response::failure(
                Some("System error"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}

// Reset password function - verifies OTP and updates password
pub async fn reset_password(
    ValidatedJson(payload): ValidatedJson<ResetPasswordDto>,
) -> impl IntoResponse {
    // Check if user exists
    if !exist_email(&payload.email).await {
        let error_response = serde_json::json!({
            "email": "No account found with this email address"
        });
        return api_response::failure(
            Some("Email not found"),
            Some(error_response),
            Some(StatusCode::NOT_FOUND),
        );
    }

    // Verify and consume OTP
    match verify_and_consume_otp(&payload.email, &payload.otp).await {
        Ok(true) => {
            // OTP is valid, proceed with password update
            match update_user_password(&payload.email, &payload.new_password).await {
                Ok(true) => {
                    // Password updated successfully
                    // Invalidate all existing tokens for security
                    if let Err(e) = invalidate_all_user_tokens(&payload.email).await {
                        println!("WARNING: Failed to invalidate tokens after password reset: {}", e);
                    }

                    api_response::success(
                        Some("Password reset successful"),
                        Some(serde_json::json!({
                            "message": "Your password has been reset successfully",
                            "email": payload.email,
                            "note": "All existing sessions have been invalidated. Please log in with your new password."
                        })),
                        Some(StatusCode::OK),
                    )
                }
                Ok(false) => {
                    let error_response = serde_json::json!({
                        "system": "User not found during password update"
                    });
                    api_response::failure(
                        Some("Password reset failed"),
                        Some(error_response),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                }
                Err(e) => {
                    println!("ERROR: Failed to update password: {}", e);
                    let error_response = serde_json::json!({
                        "system": "Failed to update password. Please try again."
                    });
                    api_response::failure(
                        Some("Password reset failed"),
                        Some(error_response),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                }
            }
        }
        Ok(false) => {
            // Invalid or expired OTP
            let error_response = serde_json::json!({
                "otp": "Invalid or expired OTP. Please request a new password reset."
            });
            api_response::failure(
                Some("Invalid OTP"),
                Some(error_response),
                Some(StatusCode::BAD_REQUEST),
            )
        }
        Err(e) => {
            println!("ERROR: Failed to verify OTP: {}", e);
            let error_response = serde_json::json!({
                "system": "Unable to verify OTP. Please try again."
            });
            api_response::failure(
                Some("System error"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}
