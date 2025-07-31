use crate::database::users::Model as UserModel;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserResource {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&UserModel> for UserResource {
    fn from(user: &UserModel) -> Self {
        Self {
            id: user.id.clone(),
            name: user.name.clone(),
            email: user.email.clone(),
            phone: user.phone.clone(),
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.to_string(),
        }
    }
}
