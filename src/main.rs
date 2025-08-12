use axum_template::{config, run};
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Connect to the database using SeaORM with retry/backoff to better handle
    // external managed databases (e.g., Supabase) and transient network issues inside containers.
    let db_url = &*crate::config::database::DB_URL;
    let max_attempts: u32 = std::env::var("DB_CONNECT_MAX_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let base_delay_ms: u64 = std::env::var("DB_CONNECT_BASE_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);

    let mut db_opt: Option<DatabaseConnection> = None;
    for attempt in 1..=max_attempts {
        match Database::connect(db_url).await {
            Ok(conn) => {
                db_opt = Some(conn);
                break;
            }
            Err(_) => {
                let delay = base_delay_ms * attempt as u64;
                sleep(Duration::from_millis(delay)).await;
            }
        }
    }

    let db = db_opt.unwrap_or_else(|| {
        panic!("Failed to connect to database after {max_attempts} attempts. Check networking, sslmode, host, and credentials.")
    });

    // Run database migrations automatically
    if let Err(e) = Migrator::up(&db, None).await {
        eprintln!("❌ Migration failed: {e}");
        std::process::exit(1);
    }

    // Initialize the Apalis job queue
    if let Err(e) = axum_template::utils::job_queue::init_job_queue().await {
        eprintln!("❌ Failed to initialize job queue: {e}");
        std::process::exit(1);
    }

    // Start the Apalis email worker in the background
    tokio::spawn(async {
        if let Err(e) = axum_template::utils::job_queue::start_email_worker().await {
            eprintln!("❌ Email worker error: {e}");
        }
    });

    run(db).await;
}
