use crate::controllers::{self};
use axum::Router;

pub fn create_routes() -> Router {
    Router::new().nest("/users", controllers::user_controller::routes())
}
