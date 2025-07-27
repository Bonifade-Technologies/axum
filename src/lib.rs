use crate::routes::handlers;
use sea_orm::{Database, DatabaseConnection};

pub mod config;
pub mod controllers;
pub mod models;
pub mod routes;
pub mod utils;
pub mod views;

pub async fn run() {
    // import the main route file here
    let app = handlers::app_router();

    // Connect to the database using SeaORM

    let db: DatabaseConnection = Database::connect(&*crate::config::database::DB_URL)
        .await
        .expect("Failed to connect to database");

    // Use APP_URL and APP_PORT static variables from config/database.rs
    let addr = format!(
        "{}:{}",
        *crate::config::database::APP_URL,
        *crate::config::database::APP_PORT
    );

    println!("Starting server at {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
