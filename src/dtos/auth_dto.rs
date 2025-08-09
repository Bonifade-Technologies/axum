use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(default)]
pub struct SignupDto {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(
        email(message = "invalid email"),
        custom(function = "validate_unique_email")
    )]
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

impl Default for SignupDto {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            password_confirmation: String::new(),
            phone: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(default)]
pub struct LoginDto {
    #[validate(
        email(message = "invalid email"),
        custom(function = "validate_existing_email")
    )]
    #[validate(length(min = 1, message = "email is required"))]
    pub email: String,

    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

impl Default for LoginDto {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
        }
    }
}

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

fn validate_unique_email(email: &str) -> Result<(), ValidationError> {
    if email == "ade@abc.com" {
        // the value of the email will automatically be added later
        return Err(
            ValidationError::new("email_taken").with_message("Email is already taken".into())
        );
    }

    Ok(())
}

fn validate_existing_email(email: &str) -> Result<(), ValidationError> {
    if email != "ade@abc.com" {
        // the value of the email will automatically be added later
        return Err(ValidationError::new("terrible_email"));
    }

    Ok(())
}
