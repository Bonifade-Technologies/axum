use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = sea_orm_migration::sea_orm::Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&conn, None).await.expect("Migration failed");
    println!("Migrations applied successfully");
}
