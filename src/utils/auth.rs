use crate::{config::database::db_connection, database::users as user};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
extern crate bcrypt;
use bcrypt::{hash, verify, DEFAULT_COST};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

// Helper functions for password hashing
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

pub fn verify_password(input: &str, stored: &str) -> bool {
    verify(input, stored).unwrap_or(false)
}

// JWT token generation
pub fn generate_jwt_token(email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now_timestamp = chrono::Utc::now().timestamp() as usize;
    let exp = now_timestamp + 24 * 60 * 60; // 24 hours from now

    let claims = Claims {
        sub: email.to_string(),
        exp,
        iat: now_timestamp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(crate::config::JWT_SECRET.as_bytes()),
    )
}

pub async fn unique_email(email: &str) -> bool {
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(_)) = existing_user {
        return false;
    }

    true
}

pub async fn exist_email(email: &str) -> bool {
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .filter(user::Column::DeletedAt.is_null())
        .one(&db_connection().await)
        .await;

    if let Ok(Some(_)) = existing_user {
        return true;
    }

    false
}
