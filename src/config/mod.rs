use once_cell::sync::Lazy;
use std::env;
pub mod database;
pub mod redis;

// JWT constants
pub const JWT_SECRET: Lazy<String> = Lazy::new(|| env::var("JWT_SECRET").unwrap());
