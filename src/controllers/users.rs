use crate::database::users as user;
use crate::dtos::auth_dto::SignupDto;
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cuid2;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserCreateRequest {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserUpdateRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
}

pub async fn create_user(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<SignupDto>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().naive_utc();
    let user = user::ActiveModel {
        id: Set(cuid2::create_id()),
        name: Set(payload.name),
        email: Set(payload.email),
        phone: Set(Some(payload.phone)),
        password: Set(payload.password),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };
    let res = user.insert(&db).await;
    match res {
        Ok(user) => {
            let resource = UserResource::from(&user);
            api_response::success(
                Some("User created"),
                Some(resource),
                Some(StatusCode::CREATED),
            )
        }
        Err(e) => api_response::failure(
            Some("Failed to create user"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

pub async fn get_user(
    State(db): State<DatabaseConnection>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = user::Entity::find_by_id(id).one(&db).await;
    match res {
        Ok(Some(user)) => {
            let resource = UserResource::from(&user);
            api_response::success(Some("User found"), Some(resource), None)
        }
        Ok(None) => api_response::failure(
            Some("User not found"),
            None::<String>,
            Some(StatusCode::NOT_FOUND),
        ),
        Err(e) => api_response::failure(
            Some("Failed to fetch user"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

pub async fn list_users(State(db): State<DatabaseConnection>) -> impl IntoResponse {
    let res = user::Entity::find().all(&db).await;
    match res {
        Ok(users) => {
            let resources: Vec<UserResource> = users.iter().map(UserResource::from).collect();
            api_response::success(Some("All users"), Some(resources), None)
        }
        Err(e) => api_response::failure(
            Some("Failed to fetch users"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

pub async fn update_user(
    State(db): State<DatabaseConnection>,
    Path(id): Path<String>,
    Json(payload): Json<UserUpdateRequest>,
) -> impl IntoResponse {
    let res = user::Entity::find_by_id(id.clone()).one(&db).await;
    match res {
        Ok(Some(user)) => {
            let mut active: user::ActiveModel = user.into();
            if let Some(name) = payload.name {
                active.name = Set(name);
            }
            if let Some(email) = payload.email {
                active.email = Set(email);
            }
            if let Some(phone) = payload.phone {
                active.phone = Set(Some(phone));
            }
            if let Some(password) = payload.password {
                active.password = Set(password);
            }
            active.updated_at = Set(chrono::Utc::now().naive_utc());
            match active.update(&db).await {
                Ok(user) => {
                    let resource = UserResource::from(&user);
                    api_response::success(Some("User updated"), Some(resource), None)
                }
                Err(e) => api_response::failure(
                    Some("Failed to update user"),
                    Some(e.to_string()),
                    Some(StatusCode::INTERNAL_SERVER_ERROR),
                ),
            }
        }
        Ok(None) => api_response::failure(
            Some("User not found"),
            None::<String>,
            Some(StatusCode::NOT_FOUND),
        ),
        Err(e) => api_response::failure(
            Some("Failed to fetch user"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}

pub async fn delete_user(
    State(db): State<DatabaseConnection>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let res = user::Entity::find_by_id(id.clone()).one(&db).await;
    match res {
        Ok(Some(user)) => {
            let mut active: user::ActiveModel = user.into();
            active.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
            match active.update(&db).await {
                Ok(user) => {
                    let resource = UserResource::from(&user);
                    api_response::success(Some("User deleted"), Some(resource), None)
                }
                Err(e) => api_response::failure(
                    Some("Failed to delete user"),
                    Some(e.to_string()),
                    Some(StatusCode::INTERNAL_SERVER_ERROR),
                ),
            }
        }
        Ok(None) => api_response::failure(
            Some("User not found"),
            None::<String>,
            Some(StatusCode::NOT_FOUND),
        ),
        Err(e) => api_response::failure(
            Some("Failed to fetch user"),
            Some(e.to_string()),
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        ),
    }
}
