use super::{
    error::LoginServiceError,
    model::{LoginErrorResponse, LoginRequest, LoginResponse},
};
use crate::{configuration::Settings, token::generate_token, user::Password};
use poem::web::Data;
use poem_openapi::{payload::Json, ApiResponse, OpenApi};
use sqlx::SqlitePool;
use validator::Validate;

#[derive(ApiResponse)]
enum LoginUserResponse {
    /// Returns when the user is successfully created.
    #[oai(status = 200)]
    Created(Json<LoginResponse>),
}

#[derive(ApiResponse)]
pub enum LoginUserResponseError {
    /// Indicates a validation error occurred during user login (e.g., missing email or password).
    #[oai(status = 400)]
    BadRequest(Json<LoginErrorResponse>),
    /// Indicates that the provided email address is not associated with a registered user.
    #[oai(status = 404)]
    NotFound(Json<LoginErrorResponse>),
    /// Indicates that the provided credentials (email and password) are incorrect.
    #[oai(status = 401)]
    Unauthorized(Json<LoginErrorResponse>),
    /// Indicates an internal server error occurred during user login. The error message provides a general indication of the problem.
    #[oai(status = 500)]
    InternalServerError(Json<LoginErrorResponse>),
}

pub struct LoginApi;

#[OpenApi]
impl LoginApi {
    /// Authenticates an existing user using their email and password.
    ///
    /// ## Request
    /// - `POST /login`
    /// - Content-Type: `application/json`
    /// - Body:
    ///    - `email`: The registered email of the user.
    ///    - `password`: The user's password.
    ///
    /// ## Responses
    /// - **200 OK**: Returns a `LoginResponse` containing the authentication token.
    /// - **400 Bad Request**: Returned if validation errors occur (e.g., invalid email or missing fields).
    /// - **401 Unauthorized**: Returned if the email or password is incorrect.
    /// - **404 Not Found**: Returned if the user does not exist in the database.
    /// - **500 Internal Server Error**: Returned if a server error occurs while processing the request.
    #[oai(path = "/login", method = "post")]
    async fn login(
        &self,
        pool: Data<&SqlitePool>,
        input: Json<LoginRequest>,
        settings: Data<&Settings>,
    ) -> Result<LoginUserResponse, LoginUserResponseError> {
        log::info!("Validating login request for email: {}", input.email);
        input
            .validate()
            .map_err(LoginServiceError::ValidationError)?;

        log::info!(
            "Fetching password hash from the database for email: {}",
            input.email
        );
        let password = sqlx::query_as!(
            Password,
            r#"
                SELECT password_hash as "hash"
                FROM users 
                WHERE email = ?
                "#,
            input.email
        )
        .fetch_one(pool.0)
        .await
        .map_err(LoginServiceError::DatabaseError)?;

        log::info!("Verifying password for email: {}", input.email);
        let matched = password
            .verify(&input.password)
            .map_err(|e| LoginServiceError::PasswordVerificationError(e.to_string()))?;

        if !matched {
            log::warn!("Password mismatch for email: {}", input.email);
            return Err(LoginUserResponseError::Unauthorized(Json(
                LoginErrorResponse {
                    message: "Password mismatched".to_string(),
                    code: 401,
                },
            )));
        }

        log::info!("Generating authentication token for email: {}", input.email);
        let token = generate_token(&input.email, &settings.0.jwt)
            .map_err(|e| LoginServiceError::TokenError(e.to_string()))?;

        log::info!("Login successful for email: {}", input.email);
        Ok(LoginUserResponse::Created(Json(LoginResponse::new(token))))
    }
}
