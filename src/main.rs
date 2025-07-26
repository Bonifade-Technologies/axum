use axum::{routing::get, Router};
use std::net::SocketAddr;

pub mod controllers;
pub mod models;
pub mod views;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_world));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}
