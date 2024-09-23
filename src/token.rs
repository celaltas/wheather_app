use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::configuration::JwtSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
    iss: String,
}

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("JWT encoding error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Invalid TTL: {0}")]
    InvalidTTL(i64),
    #[error("Token validation error: {0}")]
    ValidationError(String),
}

fn generate(
    user_email: String,
    ttl_minutes: i64,
    secret: &str,
    issuer: &str,
) -> Result<String, TokenError> {
    if ttl_minutes <= 0 {
        return Err(TokenError::InvalidTTL(ttl_minutes));
    }
    let now = Utc::now();
    let expiration = now + Duration::minutes(ttl_minutes);

    let claims = Claims {
        sub: user_email,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
        iss: issuer.to_string(),
    };

    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    let decoded_secret = general_purpose::STANDARD.decode(secret)?;
    let token = encode(&header, &claims, &EncodingKey::from_secret(&decoded_secret))?;
    Ok(token)
}

pub fn generate_token(user_email: &str, settings: &JwtSettings) -> Result<String, TokenError> {
    generate(
        user_email.to_string(),
        settings.expire_min,
        &settings.secret,
        "celal.tas.weather.app.com",
    )
}

pub fn verify_token(token: &str, settings: &JwtSettings) -> Result<Claims, TokenError> {
    verify(token, &settings.secret)
}

fn verify(token: &str, secret: &str) -> Result<Claims, TokenError> {
    let decoded_secret = general_purpose::STANDARD.decode(secret)?;
    let decoding_key = DecodingKey::from_secret(&decoded_secret);

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["celal.tas.weather.app.com"]);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| TokenError::ValidationError(e.to_string()))?;

    let claims = token_data.claims;

    let current_time = Utc::now().timestamp();
    if claims.exp < current_time {
        return Err(TokenError::ValidationError("Token has expired".to_string()));
    }

    Ok(claims)
}
