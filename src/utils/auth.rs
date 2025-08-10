use crate::{
    config::database::db_connection, config::redis::redis_client, database::users as user,
    resources::user_resource::UserResource,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
extern crate bcrypt;
use bcrypt::{hash, verify, DEFAULT_COST};
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

// Cache TTL constants
const USER_CACHE_TTL: u64 = 7 * 24 * 60 * 60; // 7 days for user data
const SESSION_TTL: u64 = 24 * 60 * 60; // 24 hours for sessions
const ACTIVE_USER_TTL: u64 = 30 * 24 * 60 * 60; // 30 days for very active users

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

// Complete cached user data including password hash
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedUser {
    pub user_resource: UserResource,
    pub password_hash: String,
}

// Helper functions for password hashing
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

pub fn verify_password(input: &str, stored: &str) -> bool {
    verify(input, stored).unwrap_or(false)
}

// JWT token generation
pub fn generate_jwt_token(email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now_timestamp = chrono::Utc::now().timestamp() as usize;
    let exp = now_timestamp + 24 * 60 * 60;

    let claims = Claims {
        sub: email.to_string(),
        exp,
        iat: now_timestamp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(crate::config::JWT_SECRET.as_bytes()),
    )
}

pub async fn unique_email(email: &str) -> bool {
    let client = redis_client();
    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{}", email);
        let exists: Result<bool, redis::RedisError> = conn.exists(&redis_key).await;
        if let Ok(true) = exists {
            return false;
        }
    }

    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(_)) = existing_user {
        return false;
    }

    true
}

pub async fn exist_email(email: &str) -> bool {
    let client = redis_client();
    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{}", email);
        let exists: Result<bool, redis::RedisError> = conn.exists(&redis_key).await;
        if let Ok(true) = exists {
            return true;
        }
    }

    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(_)) = existing_user {
        return true;
    }

    false
}

// Smart cache functions with sliding window TTL
pub async fn get_user_from_cache_or_db(email: &str) -> Option<UserResource> {
    if let Some(cached_user) = get_complete_user_from_cache_or_db(email).await {
        return Some(cached_user.user_resource);
    }
    None
}

// Get complete user data including password hash from cache or database
pub async fn get_complete_user_from_cache_or_db(email: &str) -> Option<CachedUser> {
    let client = redis_client();

    // Try Redis first
    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{}", email);

        // Check if user exists in cache
        let cached_user: Result<String, redis::RedisError> = conn.get(&redis_key).await;

        if let Ok(user_json) = cached_user {
            // User found in cache - extend TTL (sliding window)
            let _: Result<(), redis::RedisError> =
                conn.expire(&redis_key, USER_CACHE_TTL as i64).await;

            // Parse and return cached user with password
            if let Ok(cached_user) = serde_json::from_str::<CachedUser>(&user_json) {
                println!("✅ Cache HIT for user: {}", email);
                return Some(cached_user);
            }
        }
    }

    println!("💾 Cache MISS for user: {} - fetching from database", email);

    // Not in cache or cache failed - fetch from database
    let db_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(user_model)) = db_user {
        let user_resource = UserResource::from(&user_model);
        let cached_user = CachedUser {
            user_resource: user_resource.clone(),
            password_hash: user_model.password.clone(),
        };

        // Store complete user data in cache for future requests
        cache_complete_user_data(email, &cached_user).await;

        return Some(cached_user);
    }

    None
}

pub async fn cache_complete_user_data(email: &str, cached_user: &CachedUser) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        if let Ok(user_json) = serde_json::to_string(cached_user) {
            let redis_key = format!("user:{}", email);

            // Store with smart TTL based on user activity
            let ttl = get_smart_ttl_for_user(email).await;
            let _: Result<(), redis::RedisError> = conn.set_ex(&redis_key, user_json, ttl).await;

            println!(
                "💾 Cached complete user data for: {} with TTL: {} seconds",
                email, ttl
            );
        }
    }
}

pub async fn cache_user_data(email: &str, user: &UserResource) {
    // This is kept for backward compatibility, but we recommend using cache_complete_user_data
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        if let Ok(user_json) = serde_json::to_string(user) {
            let redis_key = format!("user_basic:{}", email);

            // Store with smart TTL based on user activity
            let ttl = get_smart_ttl_for_user(email).await;
            let _: Result<(), redis::RedisError> = conn.set_ex(&redis_key, user_json, ttl).await;
        }
    }
}

// Authenticate user with cached data (no DB call needed!)
pub async fn authenticate_user(email: &str, password: &str) -> Option<UserResource> {
    if let Some(cached_user) = get_complete_user_from_cache_or_db(email).await {
        if verify_password(password, &cached_user.password_hash) {
            // Increment activity for smart TTL
            increment_user_activity(email).await;

            println!("🔐 Password verified from cache for user: {}", email);
            return Some(cached_user.user_resource);
        } else {
            println!("❌ Invalid password for user: {}", email);
        }
    }
    None
}

async fn get_smart_ttl_for_user(email: &str) -> u64 {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let activity_key = format!("activity:{}", email);
        let login_count: Result<i64, redis::RedisError> = conn.get(&activity_key).await;

        match login_count {
            Ok(count) if count > 10 => ACTIVE_USER_TTL, // Very active user - 30 days
            Ok(count) if count > 3 => USER_CACHE_TTL,   // Regular user - 7 days
            _ => SESSION_TTL,                           // New/inactive user - 24 hours
        }
    } else {
        USER_CACHE_TTL // Default to 7 days if Redis unavailable
    }
}

pub async fn increment_user_activity(email: &str) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let activity_key = format!("activity:{}", email);

        // Increment activity counter with sliding window TTL
        let _: Result<i64, redis::RedisError> = conn.incr(&activity_key, 1).await;
        let _: Result<(), redis::RedisError> =
            conn.expire(&activity_key, ACTIVE_USER_TTL as i64).await;
    }
}

// Invalidate all existing tokens for a user (for secure single-session login)
pub async fn invalidate_all_user_tokens(email: &str) -> Result<i32, redis::RedisError> {
    let client = redis_client();
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Get all token keys
    let token_pattern = "token:*";
    let all_token_keys: Vec<String> = conn.keys(token_pattern).await?;

    let mut invalidated_count = 0;

    // Check each token to see if it belongs to this user
    for token_key in all_token_keys {
        let stored_email: Result<String, redis::RedisError> = conn.get(&token_key).await;

        if let Ok(stored_email) = stored_email {
            if stored_email == email {
                // Delete this token
                let deleted: i32 = conn.del(&token_key).await?;
                invalidated_count += deleted;
            }
        }
    }

    Ok(invalidated_count)
}

// OTP (One-Time Password) functions
use rand::Rng;

// Generate a 6-digit OTP
pub fn generate_otp() -> String {
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(100000..999999))
}

// Store OTP in Redis with 10-minute expiration
pub async fn store_otp(email: &str, otp: &str) -> Result<(), redis::RedisError> {
    let client = redis_client();
    let mut conn = client.get_multiplexed_async_connection().await?;

    let otp_key = format!("otp:{}", email);
    let otp_expiry = 10 * 60; // 10 minutes in seconds

    conn.set_ex::<_, _, ()>(otp_key, otp, otp_expiry).await?;
    Ok(())
}

// Verify OTP and remove it if valid
pub async fn verify_and_consume_otp(
    email: &str,
    provided_otp: &str,
) -> Result<bool, redis::RedisError> {
    let client = redis_client();
    let mut conn = client.get_multiplexed_async_connection().await?;

    let otp_key = format!("otp:{}", email);

    // Get the stored OTP
    let stored_otp: Option<String> = conn.get(&otp_key).await?;

    match stored_otp {
        Some(stored) => {
            if stored == provided_otp {
                // OTP is valid, remove it (consume it)
                let _: i32 = conn.del(&otp_key).await?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false), // OTP not found or expired
    }
}

// Update user password in both database and cache
pub async fn update_user_password(
    email: &str,
    new_password: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let hashed_password = hash_password(new_password);

    // Update in database
    let db = db_connection().await;

    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    // Find the user
    let user_result = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db)
        .await?;

    if let Some(user_model) = user_result {
        // Update password
        let mut user_active: user::ActiveModel = user_model.into();
        user_active.password = Set(hashed_password.clone());
        user_active.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_user = user_active.update(&db).await?;

        // Update cache with new password hash
        if let Some(cached_user) = get_complete_user_from_cache_or_db(email).await {
            let mut updated_cached_user = cached_user;
            updated_cached_user.password_hash = hashed_password;
            updated_cached_user.user_resource.updated_at = updated_user.updated_at.to_string();

            // Re-cache the updated user data
            cache_complete_user_data(email, &updated_cached_user).await;
        }

        Ok(true)
    } else {
        Ok(false) // User not found
    }
}
