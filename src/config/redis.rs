use once_cell::sync::Lazy;
use redis::Client;
use std::env;

pub static REDIS_URL: Lazy<String> =
    Lazy::new(|| env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string()));

pub fn redis_client() -> Client {
    Client::open(REDIS_URL.as_str()).expect("Failed to create Redis client")
}
