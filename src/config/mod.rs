use once_cell::sync::Lazy;
use std::env;
pub mod database;
pub mod redis;

// JWT constants with fallback
pub const JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("Warning: JWT_SECRET not found in environment, using default (CHANGE IN PRODUCTION!)");
        "default_jwt_secret_change_in_production_this_is_not_secure".to_string()
    })
});
