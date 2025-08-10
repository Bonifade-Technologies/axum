// tests/rate_limit_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode},
        Router,
    };
    use std::time::Duration;
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let limiter = Arc::new(RateLimiter::new(
            RateLimitConfig::new(2, Duration::from_secs(1)) // 2 requests per second for testing
        ));

        Router::new()
            .route("/test", axum::routing::get(|| async { "success" }))
            .layer(axum::middleware::from_fn(rate_limit_middleware(limiter, IpKeyExtractor)))
    }

    #[tokio::test]
    async fn test_rate_limit_allows_requests_within_limit() {
        let app = create_test_app().await;

        // First request should succeed
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Second request should also succeed
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_requests_over_limit() {
        let app = create_test_app().await;

        // Make requests up to the limit
        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/test")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Next request should be rate limited
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5, 1.0); // 5 tokens, 1 token/second refill

        // Consume all tokens
        for _ in 0..5 {
            assert!(bucket.try_consume(1.0));
        }
        
        // Should be out of tokens
        assert!(!bucket.try_consume(1.0));

        // Wait for refill
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should have tokens again
        assert!(bucket.try_consume(1.0));
        assert!(bucket.try_consume(1.0));
    }
}

// config/rate_limits.rs - Configuration management
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    pub global: RateLimitConfigSerde,
    pub auth: RateLimitConfigSerde,
    pub api: RateLimitConfigSerde,
    pub delete: RateLimitConfigSerde,
    pub custom: HashMap<String, RateLimitConfigSerde>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfigSerde {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub algorithm: String, // "token_bucket", "fixed_window", "sliding_window"
    pub burst_size: Option<u32>,
}

impl From<RateLimitConfigSerde> for RateLimitConfig {
    fn from(config: RateLimitConfigSerde) -> Self {
        let algorithm = match config.algorithm.as_str() {
            "token_bucket" => Algorithm::TokenBucket,
            "fixed_window" => Algorithm::FixedWindow,
            "sliding_window" => Algorithm::SlidingWindow,
            _ => Algorithm::TokenBucket,
        };

        RateLimitConfig {
            max_requests: config.max_requests,
            window_duration: Duration::from_secs(config.window_seconds),
            algorithm,
            burst_size: config.burst_size,
        }
    }
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            global: RateLimitConfigSerde {
                max_requests: 10,
                window_seconds: 1,
                algorithm: "token_bucket".to_string(),
                burst_size: Some(20),
            },
            auth: RateLimitConfigSerde {
                max_requests: 5,
                window_seconds: 300,
                algorithm: "fixed_window".to_string(),
                burst_size: None,
            },
            api: RateLimitConfigSerde {
                max_requests: 1000,
                window_seconds: 3600,
                algorithm: "token_bucket".to_string(),
                burst_size: Some(100),
            },
            delete: RateLimitConfigSerde {
                max_requests: 3,
                window_seconds: 60,
                algorithm: "fixed_window".to_string(),
                burst_size: None,
            },
            custom: HashMap::new(),
        }
    }
}

// Load configuration from file or environment
impl RateLimitSettings {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let settings: RateLimitSettings = toml::from_str(&content)?;
        Ok(settings)
    }

    pub fn from_env() -> Self {
        let mut settings = Self::default();
        
        // Override with environment variables
        if let Ok(val) = std::env::var("RATE_LIMIT_GLOBAL_MAX") {
            if let Ok(max) = val.parse::<u32>() {
                settings.global.max_requests = max;
            }
        }

        if let Ok(val) = std::env::var("RATE_LIMIT_AUTH_MAX") {
            if let Ok(max) = val.parse::<u32>() {
                settings.auth.max_requests = max;
            }
        }

        settings
    }
}

// Monitoring and metrics
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct RateLimitMetrics {
    pub total_requests: AtomicU64,
    pub blocked_requests: AtomicU64,
    pub avg_response_time: AtomicU64,
}

impl RateLimitMetrics {
    pub fn record_request(&self, blocked: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if blocked {
            self.blocked_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get_block_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        let blocked = self.blocked_requests.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            blocked as f64 / total as f64
        }
    }
}

// Enhanced rate limiter with metrics
#[derive(Debug)]
pub struct EnhancedRateLimiter {
    inner: RateLimiter,
    metrics: Arc<RateLimitMetrics>,
}

impl EnhancedRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            inner: RateLimiter::new(config),
            metrics: Arc::new(RateLimitMetrics::default()),
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let start = std::time::Instant::now();
        let result = self.inner.check_rate_limit(key).await;
        let blocked = result.is_err();
        
        self.metrics.record_request(blocked);
        
        result
    }

    pub fn get_metrics(&self) -> &RateLimitMetrics {
        &self.metrics
    }
}

// Production-ready configuration with Redis backend (optional)
#[cfg(feature = "redis")]
mod redis_backend {
    use redis::{AsyncCommands, Client};
    use std::sync::Arc;

    pub struct RedisRateLimiter {
        client: Arc<Client>,
        config: RateLimitConfig,
        key_prefix: String,
    }

    impl RedisRateLimiter {
        pub fn new(redis_url: &str, config: RateLimitConfig) -> Result<Self, redis::RedisError> {
            let client = Client::open(redis_url)?;
            Ok(Self {
                client: Arc::new(client),
                config,
                key_prefix: "rate_limit:".to_string(),
            })
        }

        pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
            let mut conn = self.client.get_async_connection().await
                .map_err(|_| RateLimitError::Configuration)?;

            let redis_key = format!("{}{}", self.key_prefix, key);
            let window_seconds = self.config.window_duration.as_secs();

            // Sliding window log implementation with Redis
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let window_start = now - window_seconds;

            // Remove old entries
            let _: () = conn.zremrangebyscore(&redis_key, 0, window_start as f64).await
                .map_err(|_| RateLimitError::Configuration)?;

            // Count current requests
            let count: u64 = conn.zcard(&redis_key).await
                .map_err(|_| RateLimitError::Configuration)?;

            if count >= self.config.max_requests as u64 {
                // Get the oldest request time to calculate retry_after
                let oldest: Option<f64> = conn.zrange(&redis_key, 0, 0).await.ok()
                    .and_then(|v: Vec<String>| v.first().and_then(|s| s.parse().ok()));

                let retry_after = if let Some(oldest_time) = oldest {
                    (oldest_time as u64 + window_seconds).saturating_sub(now)
                } else {
                    window_seconds
                };

                return Err(RateLimitError::Exceeded { retry_after });
            }

            // Add current request
            let _: () = conn.zadd(&redis_key, now, now).await
                .map_err(|_| RateLimitError::Configuration)?;

            // Set expiration
            let _: () = conn.expire(&redis_key, window_seconds as usize).await
                .map_err(|_| RateLimitError::Configuration)?;

            Ok(())
        }
    }
}

// Example configuration file (config/rate_limits.toml)
const EXAMPLE_CONFIG: &str = r#"
[global]
max_requests = 10
window_seconds = 1
algorithm = "token_bucket"
burst_size = 20

[auth]
max_requests = 5
window_seconds = 300
algorithm = "fixed_window"

[api]
max_requests = 1000
window_seconds = 3600
algorithm = "token_bucket"
burst_size = 100

[delete]
max_requests = 3
window_seconds = 60
algorithm = "fixed_window"

[custom.upload]
max_requests = 5
window_seconds = 60
algorithm = "token_bucket"

[custom.search]
max_requests = 100
window_seconds = 60
algorithm = "sliding_window"
"#;