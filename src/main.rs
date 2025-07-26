use axum_template::run;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    run().await;
}
