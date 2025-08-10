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
    println!(
        "🔄 [Apalis Worker] Processing password reset success email job for: {}",
        job.email
    );

    // Send the HTML email
    match send_password_reset_success_email(&job.email, &job.name, &job.reset_time).await {
        Ok(_) => {
            println!(
                "✅ [Apalis Worker] Password reset success email sent to: {}",
                job.email
            );
            Ok(())
        }
        Err(e) => {
            println!(
                "❌ [Apalis Worker] Failed to send password reset success email to {}: {}",
                job.email, e
            );
            Err(Error::Failed(std::sync::Arc::new(e)))
        }
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

    println!("🔧 [Apalis] Testing connection to Redis at: {}", redis_url);

    // Test the connection
    let _conn = apalis_redis::connect(redis_url).await?;

    println!("✅ [Apalis] Redis connection test successful");
    Ok(())
}

// Queue a password reset success email job using Apalis
pub async fn queue_password_reset_success_email(
    email: &str,
    name: &str,
    reset_time: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "📤 [Apalis] Queuing password reset success email for: {}",
        email
    );

    let job = PasswordResetSuccessEmailJob {
        email: email.to_string(),
        name: name.to_string(),
        reset_time: reset_time.to_string(),
    };

    // Create storage for this operation
    let mut storage = create_redis_storage().await?;

    // Push job to Redis queue
    match storage.push(job).await {
        Ok(job_id) => {
            println!(
                "✅ [Apalis] Password reset success email queued with ID: {:?}",
                job_id
            );
            Ok(())
        }
        Err(e) => {
            println!("❌ [Apalis] Failed to queue email job: {}", e);
            Err(Box::new(e))
        }
    }
}

// Helper function to queue jobs without blocking HTTP responses
pub fn queue_password_reset_success_email_nonblocking(
    email: String,
    name: String,
    reset_time: String,
) {
    tokio::spawn(async move {
        if let Err(e) = queue_password_reset_success_email(&email, &name, &reset_time).await {
            println!(
                "❌ [Background Queue] Failed to queue password reset email for {}: {}",
                email, e
            );
        } else {
            println!("✅ [Background Queue] Password reset email queued for: {}", email);
        }
    });
}

// Generic helper for queuing any job type without blocking
pub fn spawn_job_queue<F, Fut>(job_name: &str, future: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
{
    let job_name = job_name.to_string();
    tokio::spawn(async move {
        println!("🔄 [Background Queue] Starting job: {}", job_name);
        match future().await {
            Ok(_) => println!("✅ [Background Queue] Job completed: {}", job_name),
            Err(e) => println!("❌ [Background Queue] Job failed {}: {}", job_name, e),
        }
    });
}

// Start the Apalis worker with Redis backend
pub async fn start_email_worker() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    println!(
        "🚀 [Apalis] Starting email worker with Redis backend: {}",
        redis_url
    );

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
