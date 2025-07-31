use crate::config::redis::redis_client;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};

const CACHE_TTL_SECONDS: u64 = 60 * 60 * 24;

pub async fn get_or_set_cache<T, F, Fut>(
    key: &str,
    query_params: &str,
    fetch_fn: F,
) -> redis::RedisResult<T>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let client = redis_client();
    let mut conn = client.get_multiplexed_async_connection().await?;
    let cache_key = format!("{}:{}", key, query_params);

    match conn.get::<_, Option<String>>(&cache_key).await {
        Ok(Some(cached)) => {
            if let Ok(val) = serde_json::from_str::<T>(&cached) {
                return Ok(val);
            }
        }
        Ok(None) => {}
        Err(_) => {}
    }

    let value = fetch_fn().await;
    let serialized = serde_json::to_string(&value).unwrap();

    let _: () = conn
        .set_ex(&cache_key, serialized, CACHE_TTL_SECONDS)
        .await?;

    Ok(value)
}

pub async fn invalidate_cache_by_prefix(prefix: &str) -> redis::RedisResult<()> {
    let client = redis_client();
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Get ALL keys first
    let all_keys: Vec<String> = conn.keys("*").await?;

    // Filter keys that start with the prefix
    let matching_keys: Vec<String> = all_keys
        .into_iter()
        .filter(|key| key.starts_with(prefix))
        .collect();

    if !matching_keys.is_empty() {
        let _: () = conn.del(matching_keys).await?;
    }
    Ok(())
}
