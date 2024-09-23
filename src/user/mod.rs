use bcrypt;
use chrono::prelude::*;
use serde::Deserialize;
use thiserror::Error;

pub mod login;
pub mod middleware;
pub mod register;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Password hashing error: {0}")]
    PasswordHashError(bcrypt::BcryptError),
    #[error("Password verification error: {0}")]
    PasswordVerificationError(bcrypt::BcryptError),
}

#[derive(Deserialize, sqlx::FromRow)]
struct User {
    id: i64,
    created_at: DateTime<Utc>,
    name: String,
    email: String,
    #[sqlx(flatten)]
    password: Password,
}

impl User {
    fn new(name: &str, email: &str, plaintext: &str) -> Result<Self, UserError> {
        let password = Password::new(plaintext)?;
        Ok(User {
            id: 0,
            created_at: Utc::now(),
            name: name.to_string(),
            email: email.to_string(),
            password,
        })
    }

    fn verify_password(&self, plaintext: &str) -> Result<bool, UserError> {
        self.password.verify(plaintext)
    }
}

#[derive(Deserialize, sqlx::FromRow, Debug)]
struct Password {
    hash: String,
}

impl Password {
    fn new(plaintext: &str) -> Result<Self, UserError> {
        let hash = bcrypt::hash(plaintext, 12).map_err(UserError::PasswordHashError)?;
        Ok(Password { hash })
    }

    fn verify(&self, plaintext: &str) -> Result<bool, UserError> {
        Ok(bcrypt::verify(plaintext, &self.hash).map_err(UserError::PasswordVerificationError)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("John Doe", "john@example.com", "password123");
        assert!(user.is_ok());
    }

    #[test]
    fn test_password_verification() {
        let user = User::new("Jane Doe", "jane@example.com", "password456").unwrap();
        assert!(user.verify_password("password456").unwrap());
        assert!(!user.verify_password("wrongpassword").unwrap());
    }
}
