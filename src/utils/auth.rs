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

            // Parse and return cached user
            if let Ok(user) = serde_json::from_str::<UserResource>(&user_json) {
                return Some(user);
            }
        }
    }

    // Not in cache or cache failed - fetch from database
    let db_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(user_model)) = db_user {
        let user_resource = UserResource::from(&user_model);

        // Store in cache for future requests
        cache_user_data(email, &user_resource).await;

        return Some(user_resource);
    }

    None
}

pub async fn cache_user_data(email: &str, user: &UserResource) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        if let Ok(user_json) = serde_json::to_string(user) {
            let redis_key = format!("user:{}", email);

            // Store with smart TTL based on user activity
            let ttl = get_smart_ttl_for_user(email).await;
            let _: Result<(), redis::RedisError> = conn.set_ex(&redis_key, user_json, ttl).await;
        }
    }
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

        // Increment login counter with 30-day expiry
        let _: Result<i64, redis::RedisError> = conn.incr(&activity_key, 1).await;
        let _: Result<(), redis::RedisError> = conn.expire(&activity_key, 30 * 24 * 60 * 60).await;
    }
}
