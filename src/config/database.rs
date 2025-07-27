use once_cell::sync::Lazy;
use std::env;

pub static APP_URL: Lazy<String> = Lazy::new(|| env::var("APP_URL").unwrap());
pub static APP_PORT: Lazy<String> = Lazy::new(|| env::var("APP_PORT").unwrap());
pub static DB_URL: Lazy<String> =
    Lazy::new(|| env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
