use crate::config::redis::redis_client;
use crate::database::users as user;
use crate::resources::user_resource::UserResource;
use crate::utils::auth::Claims;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::AsyncCommands;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Get token from Authorization header
    let auth_header = request.headers().get("Authorization");
    match auth_header {
        Some(header) => {
            if let Ok(auth_str) = header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").trim();

                    // Validate JWT token
                    let token_data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(crate::config::JWT_SECRET.as_bytes()),
                        &Validation::default(),
                    );

                    if let Ok(token_data) = token_data {
                        let email = token_data.claims.sub;

                        // Verify token with Redis (for revocation support)
                        let client = redis_client();
                        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                            if let Ok(Some(user_email)) = conn
                                .get::<_, Option<String>>(format!("token:{}", token))
                                .await
                            {
                                // Ensure the email from JWT matches the one in Redis
                                if user_email == email {
                                    // Get user from Redis or fall back to database
                                    if let Ok(Some(user_json)) = conn
                                        .get::<_, Option<String>>(format!("user:{}", user_email))
                                        .await
                                    {
                                        if let Ok(user) =
                                            serde_json::from_str::<UserResource>(&user_json)
                                        {
                                            let mut req = request;
                                            req.extensions_mut().insert(user);
                                            return Ok(next.run(req).await);
                                        }
                                    }

                                    // If user not in Redis, get from database
                                    let db = request.extensions().get::<DatabaseConnection>();
                                    if let Some(db) = db {
                                        let user_result = user::Entity::find()
                                            .filter(user::Column::Email.eq(user_email))
                                            .filter(user::Column::DeletedAt.is_null())
                                            .one(db)
                                            .await;

                                        if let Ok(Some(user)) = user_result {
                                            let user_resource = UserResource::from(&user);
                                            let mut req = request;
                                            req.extensions_mut().insert(user_resource);
                                            return Ok(next.run(req).await);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }

    // Token is invalid or missing
    Err(StatusCode::UNAUTHORIZED)
}
