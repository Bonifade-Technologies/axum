use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Deserialize, Serialize, Validate, Default)]
#[serde(default)]
pub struct SignupDto {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(email(message = "invalid email"))]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,

    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,

    #[validate(must_match(other = "password", message = "passwords must match"))]
    pub password_confirmation: String,

    #[validate(length(
        min = 10,
        max = 15,
        message = "phone must be between 10 and 15 characters"
    ))]
    pub phone: String,
}

// Default derived

#[derive(Debug, Deserialize, Serialize, Validate, Default)]
#[serde(default)]
pub struct LoginDto {
    #[validate(email(message = "invalid email"))]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,

    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

// Default derived

#[derive(Debug, Deserialize, Serialize, Validate, Default)]
#[serde(default)]
pub struct ForgotPasswordDto {
    #[validate(email(message = "invalid email"))]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,
}

// Default derived

#[derive(Debug, Deserialize, Serialize, Validate, Default)]
#[serde(default)]
pub struct ResetPasswordDto {
    #[validate(email(message = "invalid email"))]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,

    #[validate(length(min = 6, max = 6, message = "OTP must be exactly 6 digits"))]
    pub otp: String,

    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub new_password: String,

    #[validate(must_match(other = "new_password", message = "passwords must match"))]
    pub confirm_password: String,
}

// Default derived

pub fn validation_errors_to_map(errors: &ValidationErrors) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (field, errs) in errors.field_errors().iter() {
        if let Some(err) = errs.first() {
            let msg = err
                .message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "invalid value".to_string());
            map.insert(field.to_string(), msg);
        }
    }
    map
}
