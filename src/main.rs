use axum_template::{config, run};
use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Connect to the database using SeaORM
    let db: DatabaseConnection = Database::connect(&*crate::config::database::DB_URL)
        .await
        .expect("Failed to connect to database");

    // Initialize the Apalis job queue
    if let Err(e) = axum_template::utils::job_queue::init_job_queue().await {
        println!("❌ Failed to initialize job queue: {}", e);
        std::process::exit(1);
    }

    // Start the Apalis email worker in the background
    tokio::spawn(async {
        if let Err(e) = axum_template::utils::job_queue::start_email_worker().await {
            println!("❌ Email worker error: {}", e);
        }
    });

    println!("🚀 Apalis email worker started for background job processing");
    run(db).await;
}
