use once_cell::sync::Lazy;
use sea_orm::{Database, DatabaseConnection};
use std::env;

pub static APP_URL: Lazy<String> = Lazy::new(|| env::var("APP_URL").unwrap());
pub static APP_PORT: Lazy<String> = Lazy::new(|| env::var("APP_PORT").unwrap());
pub static DB_URL: Lazy<String> =
    Lazy::new(|| env::var("DATABASE_URL").expect("DATABASE_URL must be set"));

pub async fn db_connection() -> DatabaseConnection {
    Database::connect(&*DB_URL)
        .await
        .expect("Failed to connect to database")
}
