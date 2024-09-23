use super::api::GetWeatherResponseError;
use super::model::WeatherErrorResponse;
use poem_openapi::payload::Json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Failed to extract IP: {0}")]
    IpExtraction(String),
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("Bad request to weather API")]
    BadRequest,
    #[error("Access to the weather API is forbidden")]
    Forbidden,
    #[error("Unknown error occurred")]
    Unknown,
}



impl From<WeatherError> for GetWeatherResponseError {
    fn from(err: WeatherError) -> Self {
        match err {
            WeatherError::BadRequest => {
                GetWeatherResponseError::BadRequest(Json(WeatherErrorResponse {
                    code: 400,
                    message: err.to_string(),
                }))
            }
            WeatherError::Forbidden => {
                GetWeatherResponseError::Forbidden(Json(WeatherErrorResponse {
                    code: 403,
                    message: err.to_string(),
                }))
            }
            _ => GetWeatherResponseError::InternalServerError(Json(WeatherErrorResponse {
                code: 500,
                message: err.to_string(),
            })),
        }
    }
}
