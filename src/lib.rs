pub mod extractors;
use sea_orm::DatabaseConnection;

pub mod config;
pub mod controllers;
pub mod database;
pub mod dtos;
pub mod models;
pub mod resources;
pub mod routes;
pub mod utils;
pub mod views;

pub async fn run(db: DatabaseConnection) {
    // import the main route file here
    let app = routes::app_router(db);

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
