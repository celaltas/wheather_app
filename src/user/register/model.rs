use chrono::prelude::*;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Object, Debug, Validate)]
pub struct RegisterRequest {
    /// Username of the user. Must be between 3 and 50 characters long.
    #[validate(length(min = 3, max = 50))]
    pub name: String,
    /// Email address of the user. Must be a valid email address.
    #[validate(email)]
    pub email: String,
    /// Password for the user. Must be between 8 and 128 characters long.
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Serialize, Object)]
pub struct RegisterResponse {
    /// Unique identifier for the user.
    pub id: i64,
    /// Username of the user.
    pub name: String,
    /// Email address of the user.
    pub email: String,
    /// Date and time the user was created.
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Object)]
pub struct RegisterErrorResponse {
    pub code: u16,
    pub message: String,
}
