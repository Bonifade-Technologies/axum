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

    run(db).await;
}
