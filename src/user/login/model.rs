use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Object, Debug, Validate)]
pub struct LoginRequest {
    /// Email address of the user. Must be a valid email address.
    #[validate(email)]
    pub email: String,

    /// Password for the user. Must be between 8 and 128 characters long.
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Serialize, Object)]
pub struct LoginResponse {
    /// Authentication token for the user.
    token: String,
}

impl LoginResponse {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Serialize, Object)]
pub struct LoginErrorResponse {
    /// The HTTP status code of the error.
    pub code: u16,
    /// A human-readable description of the error.
    pub message: String,
}
