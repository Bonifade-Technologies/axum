use crate::config::redis::redis_client;
use crate::utils::api_response;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use redis::AsyncCommands;

pub async fn clear_all_caches() -> impl IntoResponse {
    let client = redis_client();
    
    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // Get all keys
            let keys: Result<Vec<String>, redis::RedisError> = conn.keys("*").await;
            
            match keys {
                Ok(key_list) => {
                    if key_list.is_empty() {
                        return api_response::success(
                            Some("Cache is already empty"),
                            Some(serde_json::json!({"cleared_keys": 0})),
                            Some(StatusCode::OK),
                        );
                    }
                    
                    // Delete all keys
                    let deleted: Result<i32, redis::RedisError> = conn.del(&key_list).await;
                    
                    match deleted {
                        Ok(count) => {
                            api_response::success(
                                Some("All caches cleared successfully"),
                                Some(serde_json::json!({
                                    "cleared_keys": count,
                                    "cache_types_cleared": [
                                        "user_cache",
                                        "tokens", 
                                        "activity_counters",
                                        "sessions"
                                    ]
                                })),
                                Some(StatusCode::OK),
                            )
                        }
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "redis": format!("Failed to delete keys: {}", e)
                            });
                            api_response::failure(
                                Some("Cache clearing failed"),
                                Some(error_response),
                                Some(StatusCode::INTERNAL_SERVER_ERROR),
                            )
                        }
                    }
                }
                Err(e) => {
                    let error_response = serde_json::json!({
                        "redis": format!("Failed to retrieve keys: {}", e)
                    });
                    api_response::failure(
                        Some("Cache clearing failed"),
                        Some(error_response),
                        Some(StatusCode::INTERNAL_SERVER_ERROR),
                    )
                }
            }
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "redis": format!("Redis connection failed: {}", e)
            });
            api_response::failure(
                Some("Cache clearing failed"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}

pub async fn clear_user_cache(Path(email): Path<String>) -> impl IntoResponse {
    let client = redis_client();
    
    match client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // Keys to clear for a specific user
            let keys_to_clear = vec![
                format!("user:{}", email),
                format!("activity:{}", email),
            ];
            
            // Get all tokens for this user
            let token_pattern = format!("token:*");
            let all_tokens: Result<Vec<String>, redis::RedisError> = conn.keys(&token_pattern).await;
            
            let mut total_cleared = 0;
            
            // Clear user-specific keys
            for key in &keys_to_clear {
                let deleted: Result<i32, redis::RedisError> = conn.del(key).await;
                if let Ok(count) = deleted {
                    total_cleared += count;
                }
            }
            
            // Clear user's tokens
            if let Ok(tokens) = all_tokens {
                for token_key in tokens {
                    let token_email: Result<String, redis::RedisError> = conn.get(&token_key).await;
                    if let Ok(stored_email) = token_email {
                        if stored_email == email {
                            let deleted: Result<i32, redis::RedisError> = conn.del(&token_key).await;
                            if let Ok(count) = deleted {
                                total_cleared += count;
                            }
                        }
                    }
                }
            }
            
            api_response::success(
                Some("User cache cleared successfully"),
                Some(serde_json::json!({
                    "email": email,
                    "cleared_keys": total_cleared,
                    "cache_types_cleared": ["user_data", "activity", "tokens"]
                })),
                Some(StatusCode::OK),
            )
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "redis": format!("Redis connection failed: {}", e)
            });
            api_response::failure(
                Some("User cache clearing failed"),
                Some(error_response),
                Some(StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
    }
}
