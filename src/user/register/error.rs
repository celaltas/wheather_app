use poem_openapi::payload::Json;
use thiserror::Error;
use validator::ValidationErrors;
use super::{api::RegisterUserResponseError, model::RegisterErrorResponse};






#[derive(Error, Debug)]
pub enum RegisterServiceError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    #[error("User creation error: {0}")]
    UserCreationError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl From<RegisterServiceError> for RegisterUserResponseError {
    fn from(err: RegisterServiceError) -> Self {
        match err {
            RegisterServiceError::ValidationError(e) => {
                RegisterUserResponseError::BadRequest(Json(RegisterErrorResponse {
                    code: 400,
                    message: e.to_string(),
                }))
            }
            RegisterServiceError::UserCreationError(e) => {
                RegisterUserResponseError::BadRequest(Json(RegisterErrorResponse {
                    code: 400,
                    message: e,
                }))
            }
            RegisterServiceError::DatabaseError(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.code().as_deref() == Some("2067") {
                        return RegisterUserResponseError::Conflict(Json(RegisterErrorResponse {
                            code: 409,
                            message: "Email already exists".to_string(),
                        }));
                    }
                }
                RegisterUserResponseError::InternalServerError(Json(RegisterErrorResponse {
                    code: 500,
                    message: "Internal Server Error".to_string(),
                }))
            }
        }
    }
}