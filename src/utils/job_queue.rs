use apalis::prelude::*;
use apalis_redis::RedisStorage;
use serde::{Deserialize, Serialize};
use std::env;

// Job types for the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetSuccessEmailJob {
    pub email: String,
    pub name: String,
    pub reset_time: String,
}

// Job processing function for Apalis
use crate::utils::email_service::send_password_reset_success_email;

async fn process_password_reset_success_email(
    job: PasswordResetSuccessEmailJob,
    _data: Data<()>,
) -> Result<(), Error> {
    // Send the HTML email
    match send_password_reset_success_email(&job.email, &job.name, &job.reset_time).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Failed(std::sync::Arc::new(e))),
    }
}

// Helper function to create Redis storage
async fn create_redis_storage(
) -> Result<RedisStorage<PasswordResetSuccessEmailJob>, Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let conn = apalis_redis::connect(redis_url).await?;
    Ok(RedisStorage::new(conn))
}

// Initialize the Redis storage (validation only)
pub async fn init_job_queue() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Test the connection
    let _conn = apalis_redis::connect(redis_url).await?;

    Ok(())
}

// Queue a password reset success email job using Apalis
pub async fn queue_password_reset_success_email(
    email: &str,
    name: &str,
    reset_time: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let job = PasswordResetSuccessEmailJob {
        email: email.to_string(),
        name: name.to_string(),
        reset_time: reset_time.to_string(),
    };

    // Create storage for this operation
    let mut storage = create_redis_storage().await?;

    // Push job to Redis queue
    match storage.push(job).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

// Helper function to queue jobs without blocking HTTP responses
pub fn queue_password_reset_success_email_nonblocking(
    email: String,
    name: String,
    reset_time: String,
) {
    tokio::spawn(async move {
        if (queue_password_reset_success_email(&email, &name, &reset_time).await).is_err() {
            // Failed to queue email
        }
    });
}

// Generic helper for queuing any job type without blocking
pub fn spawn_job_queue<F, Fut>(job_name: &str, future: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let _job_name = job_name.to_string();
    tokio::spawn(async move { if (future().await).is_ok() {} });
}

// Start the Apalis worker with Redis backend
pub async fn start_email_worker() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Connect to Redis
    let conn = apalis_redis::connect(redis_url).await?;
    let storage = RedisStorage::new(conn);

    // Build and run the worker
    Monitor::new()
        .register({
            WorkerBuilder::new("email-worker")
                .concurrency(2) // Process up to 2 jobs concurrently
                .data(()) // No shared data needed
                .backend(storage.clone())
                .build_fn(process_password_reset_success_email)
        })
        .run()
        .await?;

    Ok(())
}
