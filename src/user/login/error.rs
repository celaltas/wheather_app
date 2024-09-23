use poem_openapi::payload::Json;
use thiserror::Error;
use validator::ValidationErrors;

use super::{api::LoginUserResponseError, model::LoginErrorResponse};

#[derive(Error, Debug)]
pub enum LoginServiceError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Password verification error: {0}")]
    PasswordVerificationError(String),
    #[error("Jwt token error: {0}")]
    TokenError(String),
}

impl From<LoginServiceError> for LoginUserResponseError {
    fn from(err: LoginServiceError) -> Self {
        match err {
            LoginServiceError::ValidationError(e) => {
                LoginUserResponseError::BadRequest(Json(LoginErrorResponse {
                    code: 400,
                    message: e.to_string(),
                }))
            }
            LoginServiceError::PasswordVerificationError(e) => {
                LoginUserResponseError::Unauthorized(Json(LoginErrorResponse {
                    code: 401,
                    message: e,
                }))
            }

            LoginServiceError::TokenError(e) => {
                LoginUserResponseError::InternalServerError(Json(LoginErrorResponse {
                    code: 500,
                    message: e,
                }))
            }

            LoginServiceError::DatabaseError(e) => {
                if let sqlx::Error::RowNotFound = e {
                    return LoginUserResponseError::NotFound(Json(LoginErrorResponse {
                        code: 404,
                        message: "User not found".to_string(),
                    }));
                }
                LoginUserResponseError::InternalServerError(Json(LoginErrorResponse {
                    code: 500,
                    message: "Internal Server Error".to_string(),
                }))
            }
        }
    }
}
