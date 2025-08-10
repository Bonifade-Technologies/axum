use crate::config::redis::redis_client;
use crate::utils::api_response;
use crate::utils::auth::{get_user_from_cache_or_db, Claims};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::AsyncCommands;

pub async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Get token from Authorization header
    let auth_header = request.headers().get("Authorization");
    match auth_header {
        Some(header) => {
            if let Ok(auth_str) = header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").trim();
                    println!("DEBUG: Token received: {}", token);

                    // Validate JWT token
                    let token_data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(crate::config::JWT_SECRET.as_bytes()),
                        &Validation::default(),
                    );

                    match token_data {
                        Ok(token_data) => {
                            let email = token_data.claims.sub.clone();
                            println!("DEBUG: JWT valid, email: {}", email);

                            // Verify token with Redis (for revocation support)
                            let client = redis_client();
                            match client.get_multiplexed_async_connection().await {
                                Ok(mut conn) => {
                                    // Check if token exists in Redis
                                    match conn
                                        .get::<_, Option<String>>(format!("token:{}", token))
                                        .await
                                    {
                                        Ok(Some(user_email)) => {
                                            println!(
                                                "DEBUG: Token found in Redis for email: {}",
                                                user_email
                                            );

                                            // Ensure the email from JWT matches the one in Redis
                                            if user_email == email {
                                                // Get real user data from cache or database
                                                if let Some(user_resource) =
                                                    get_user_from_cache_or_db(&email).await
                                                {
                                                    println!(
                                                        "DEBUG: Real user data found: {}",
                                                        user_resource.email
                                                    );
                                                    let mut req = request;
                                                    req.extensions_mut().insert(user_resource);
                                                    return Ok(next.run(req).await);
                                                } else {
                                                    println!("DEBUG: User not found in cache or database");
                                                }
                                            } else {
                                                println!(
                                                    "DEBUG: Email mismatch - JWT: {}, Redis: {}",
                                                    email, user_email
                                                );
                                            }
                                        }
                                        Ok(None) => {
                                            println!("DEBUG: Token not found in Redis");
                                        }
                                        Err(e) => {
                                            println!("DEBUG: Redis error checking token: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("DEBUG: Redis connection error: {}", e);
                                    // If Redis is down, get user directly from database
                                    if let Some(user_resource) =
                                        get_user_from_cache_or_db(&email).await
                                    {
                                        println!(
                                            "DEBUG: Fallback - Real user data found from DB: {}",
                                            user_resource.email
                                        );
                                        let mut req = request;
                                        req.extensions_mut().insert(user_resource);
                                        return Ok(next.run(req).await);
                                    } else {
                                        println!(
                                            "DEBUG: Fallback - User not found in database either"
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("DEBUG: JWT validation error: {}", e);
                        }
                    }
                } else {
                    println!("DEBUG: Authorization header doesn't start with 'Bearer '");
                }
            } else {
                println!("DEBUG: Invalid Authorization header format");
            }
        }
        None => {
            println!("DEBUG: No Authorization header found");
        }
    }

    // Token is invalid or missing - return proper JSON error response
    let error_response = serde_json::json!({
        "token": "Authentication token is required"
    });
    Ok(api_response::failure(
        Some("Unauthorized access"),
        Some(error_response),
        Some(StatusCode::UNAUTHORIZED),
    ))
}
