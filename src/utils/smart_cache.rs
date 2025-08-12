use crate::{
    config::redis::redis_client, database::users as user, resources::user_resource::UserResource,
};
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, Database, EntityTrait, QueryFilter};

// Cache TTL constants
const USER_CACHE_TTL: u64 = 7 * 24 * 60 * 60; // 7 days for user data
const SESSION_TTL: u64 = 24 * 60 * 60; // 24 hours for sessions
const ACTIVE_USER_TTL: u64 = 30 * 24 * 60 * 60; // 30 days for very active users

/// Smart cache strategy that:
/// 1. Checks Redis first (with sliding window TTL)
/// 2. Falls back to database if not cached
/// 3. Automatically caches DB results
/// 4. Extends TTL on every access
pub async fn get_user_with_smart_cache(email: &str) -> Option<UserResource> {
    let client = redis_client();

    // Try Redis first
    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{email}");

        // Check if user exists in cache
        let cached_user: Result<String, redis::RedisError> = conn.get(&redis_key).await;

        if let Ok(user_json) = cached_user {
            // User found in cache - extend TTL (sliding window)
            let new_ttl = get_smart_ttl_for_user(email).await;
            let _: Result<(), redis::RedisError> = conn.expire(&redis_key, new_ttl as i64).await;

            // Increment activity counter
            increment_user_activity(email).await;

            // Parse and return cached user
            if let Ok(user) = serde_json::from_str::<UserResource>(&user_json) {
                return Some(user);
            }
        }
    }

    // Not in cache or cache failed - fetch from database
    if let Ok(db) = Database::connect(&*crate::config::database::DB_URL).await {
        let db_user = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::DeletedAt.is_null())
            .one(&db)
            .await;

        if let Ok(Some(user_model)) = db_user {
            let user_resource = UserResource::from(&user_model);

            // Store in cache for future requests
            cache_user_with_smart_ttl(email, &user_resource).await;

            return Some(user_resource);
        }
    }

    None
}

/// Cache user data with smart TTL based on activity
pub async fn cache_user_with_smart_ttl(email: &str, user: &UserResource) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        if let Ok(user_json) = serde_json::to_string(user) {
            let redis_key = format!("user:{email}");

            // Store with smart TTL based on user activity
            let ttl = get_smart_ttl_for_user(email).await;
            let _: Result<(), redis::RedisError> = conn.set_ex(&redis_key, user_json, ttl).await;
        }
    }
}

/// Determine TTL based on user activity
async fn get_smart_ttl_for_user(email: &str) -> u64 {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let activity_key = format!("activity:{email}");
        let login_count: Result<i64, redis::RedisError> = conn.get(&activity_key).await;

        match login_count {
            Ok(count) if count > 20 => ACTIVE_USER_TTL,
            Ok(count) if count > 5 => USER_CACHE_TTL,
            Ok(_) => SESSION_TTL,
            Err(_) => USER_CACHE_TTL,
        }
    } else {
        USER_CACHE_TTL // Default to 7 days if Redis unavailable
    }
}

/// Track user login activity for smart TTL calculation
pub async fn increment_user_activity(email: &str) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let activity_key = format!("activity:{email}");

        // Increment login counter with 30-day expiry
        let _: Result<i64, redis::RedisError> = conn.incr(&activity_key, 1).await;
        let _: Result<(), redis::RedisError> = conn.expire(&activity_key, 30 * 24 * 60 * 60).await;
    }
}

/// Extend cache TTL when user is accessed (sliding window)
pub async fn extend_user_cache_ttl(email: &str) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{email}");
        let new_ttl = get_smart_ttl_for_user(email).await;

        let _: Result<(), redis::RedisError> = conn.expire(&redis_key, new_ttl as i64).await;
    }
}

/// Check if user exists (with cache benefits)
pub async fn user_exists_smart(email: &str) -> bool {
    get_user_with_smart_cache(email).await.is_some()
}

/// Clear user from cache (useful for profile updates)
pub async fn invalidate_user_cache(email: &str) {
    let client = redis_client();

    if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
        let redis_key = format!("user:{email}");
        let _: Result<(), redis::RedisError> = conn.del(&redis_key).await;
    }
}
