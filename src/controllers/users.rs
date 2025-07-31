use crate::database::users as user;
use crate::dtos::auth_dto::SignupDto;
use crate::extractors::json_extractor::ValidatedJson;
use crate::resources::user_resource::UserResource;
use crate::utils::api_response;
use crate::utils::query_params::QueryParams;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cuid2;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};

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

pub async fn list_users(
    State(db): State<DatabaseConnection>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    use sea_orm::{PaginatorTrait, QueryFilter, QueryOrder};

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let sort_by = params.sort_by.unwrap_or_else(|| "created_at".to_string());
    let sort_order = params.sort_order.unwrap_or_else(|| "desc".to_string());
    let search = params.search.clone();

    let mut query = user::Entity::find();

    // Search by name or email if search param is provided
    if let Some(ref s) = search {
        use sea_orm::sea_query::Expr;
        let search_term = format!("%{}%", s.to_lowercase());
        query = query.filter(
            Expr::cust("LOWER(name)")
                .like(&search_term)
                .or(Expr::cust("LOWER(email)").like(&search_term)),
        );
    }

    // Sorting
    query = match sort_by.as_str() {
        "name" => {
            if sort_order == "asc" {
                query.order_by_asc(user::Column::Name)
            } else {
                query.order_by_desc(user::Column::Name)
            }
        }
        "email" => {
            if sort_order == "asc" {
                query.order_by_asc(user::Column::Email)
            } else {
                query.order_by_desc(user::Column::Email)
            }
        }
        "created_at" | _ => {
            if sort_order == "asc" {
                query.order_by_asc(user::Column::CreatedAt)
            } else {
                query.order_by_desc(user::Column::CreatedAt)
            }
        }
    };

    let paginator = query.paginate(&db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page(page - 1).await;

    match users {
        Ok(users) => {
            let resources: Vec<UserResource> = users.iter().map(UserResource::from).collect();
            let pagination = crate::utils::api_response::pagination_info(page, per_page, total);
            api_response::success(
                Some("All users"),
                Some(serde_json::json!({
                    "users": resources,
                    "pagination": pagination
                })),
                None,
            )
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
