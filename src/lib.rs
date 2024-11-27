mod controllers;
mod models;
mod routes;
mod utils;
mod views;

pub async fn run() {
    let app = routes::create_routes();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        utils::constants::HOST,
        utils::constants::PORT
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}
