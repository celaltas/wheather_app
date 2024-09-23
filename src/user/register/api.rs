use crate::user::User;

use super::{
    error::RegisterServiceError,
    model::{RegisterErrorResponse, RegisterRequest, RegisterResponse},
};
use chrono::prelude::*;
use poem::web::Data;
use poem_openapi::{payload::Json, ApiResponse, OpenApi};
use sqlx::SqlitePool;
use validator::Validate;

#[derive(ApiResponse)]
pub enum RegisterUserResponse {
    /// Returns when the user is successfully created.
    #[oai(status = 201)]
    Created(Json<RegisterResponse>),
}

#[derive(ApiResponse)]
pub enum RegisterUserResponseError {
    /// Indicates a validation error occurred during user creation. The error message contains more details about the specific validation issue.
    #[oai(status = 400)]
    BadRequest(Json<RegisterErrorResponse>),
    /// Indicates that a user with the specified email address already exists in the system.
    #[oai(status = 409)]
    Conflict(Json<RegisterErrorResponse>),
    /// Indicates an internal server error occurred during user creation. The error message provides a general indication of the problem.
    #[oai(status = 500)]
    InternalServerError(Json<RegisterErrorResponse>),
}

pub struct RegisterApi;

#[OpenApi]
impl RegisterApi {
    /// Register a new user. 
    /// 
    /// ## Request
    /// - `POST /register`
    /// - Content-Type: `application/json`
    /// - Body: 
    ///    - `name`: The name of the user.
    ///    - `email`: A valid email address for the user.
    ///    - `password`: The password for the user (hashed and stored securely).
    ///
    /// ## Responses
    /// - **201 Created**: Returns a `RegisterResponse` containing the user's details upon successful creation.
    /// - **400 Bad Request**: Returned if validation errors occur (e.g., invalid email or missing fields).
    /// - **409 Conflict**: Returned if a user with the specified email already exists.
    /// - **500 Internal Server Error**: Returned if a server error occurs while processing the request.
    #[oai(path = "/register", method = "post")]
    async fn register(
        &self,
        pool: Data<&SqlitePool>,
        input: Json<RegisterRequest>,
    ) -> Result<RegisterUserResponse, RegisterUserResponseError> {

        log::info!("Validating register request for email: {}", input.email);
        input
            .validate()
            .map_err(RegisterServiceError::ValidationError)?;

        log::info!("Creating new user: {}", input.email);
        let user = User::new(&input.name, &input.email, &input.password)
            .map_err(|e| RegisterServiceError::UserCreationError(e.to_string()))?;

        log::info!("Inserting user into database: {}", input.email);
        let result = sqlx::query_as!(
            RegisterResponse,
            r#"
            INSERT INTO users (name, email, password_hash)
            VALUES (?, ?, ?)
            RETURNING id, name, email, created_at as "created_at: DateTime<Utc>"
            "#,
            user.name,
            user.email,
            user.password.hash
        )
        .fetch_one(pool.0)
        .await
        .map_err(RegisterServiceError::DatabaseError)?;

        log::info!("User successfully registered: {}", input.email);
        Ok(RegisterUserResponse::Created(Json(result)))
    }
}
