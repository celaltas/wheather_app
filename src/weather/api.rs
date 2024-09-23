use crate::{
    configuration::Settings,
    weather::error::WeatherError,
    weather::model::{WeatherErrorResponse, WeatherResponse},
};
use log::{error, info, warn};
use poem::web::{Data, RemoteAddr};
use poem_openapi::{payload::Json, ApiResponse, OpenApi};
use quick_cache::sync::Cache;
use reqwest::{Client, StatusCode};
use std::sync::Arc;

#[derive(ApiResponse)]
pub enum GetWeatherResponseError {
    /// Indicates a bad request to the weather API. The error message contains more details about the specific issue.
    #[oai(status = 400)]
    BadRequest(Json<WeatherErrorResponse>),

    /// Indicates an internal server error occurred while fetching weather data. The error message provides a general indication of the problem.
    #[oai(status = 500)]
    InternalServerError(Json<WeatherErrorResponse>),

    /// Indicates a forbidden access to the weather API. This typically occurs due to invalid credentials or rate limits.
    #[oai(status = 403)]
    Forbidden(Json<WeatherErrorResponse>),
}

#[derive(ApiResponse)]
enum GetWeatherResponse {
    /// Returns when the weather information is successfully retrieved.
    #[oai(status = 200)]
    Ok(Json<WeatherResponse>),
}

pub struct WeatherApi;

#[OpenApi]
impl WeatherApi {
    /// Retrieves current weather information based on the user's IP address.
    ///
    /// ## Request
    /// - `GET /weather`
    /// - Retrieves the weather data based on the IP address of the incoming request.
    ///
    /// ## Headers
    /// - `x-rapidapi-key`: The API key for accessing the weather data.
    /// - `x-rapidapi-host`: The API host for the weather service.
    ///
    /// ## Responses
    /// - **200 OK**: Returns a `WeatherResponse` containing current weather information.
    /// - **400 Bad Request**: If the IP address is invalid or missing, the request will fail with a bad request error.
    /// - **401 Unauthorized**: Returned when the JWT token is missing, expired, or invalid, indicating the user is not authenticated.
    /// - **403 Forbidden**: Returned when the weather API service denies access, potentially due to an invalid API key.
    /// - **500 Internal Server Error**: Returned for unknown or server-side errors while processing the request.
    ///
    /// ## Cache
    /// - The weather data is cached based on the user's IP to avoid redundant API calls and improve response times.
    #[oai(path = "/weather", method = "get")]
    async fn weather(
        &self,
        remote_addr: &RemoteAddr,
        http_client: Data<&Client>,
        conf: Data<&Settings>,
        cache: Data<&Arc<Cache<String, WeatherResponse>>>,
    ) -> Result<GetWeatherResponse, GetWeatherResponseError> {
        let ip = extract_ip(remote_addr)?;
        info!("Received weather request for IP: {}", ip);

        if let Some(cached_weather) = cache.get(&ip) {
            info!("Weather data for IP {} found in cache", ip);
            return Ok(GetWeatherResponse::Ok(Json(cached_weather.clone())));
        }

        let url = format!(
            "{}/current.json?q={}",
            &conf.wheather_api.weather_api_url, ip
        );
        info!("Fetching weather data from API for IP: {}", ip);

        let response = http_client
            .get(&url)
            .header("x-rapidapi-key", &conf.wheather_api.rapidapi_key)
            .header("x-rapidapi-host", &conf.wheather_api.rapidapi_host)
            .send()
            .await
            .map_err(WeatherError::HttpRequest)?;

        match response.status() {
            StatusCode::OK => {
                info!("Weather API responded with 200 OK for IP: {}", ip);
                let weather_data = response
                    .json::<WeatherResponse>()
                    .await
                    .map_err(WeatherError::HttpRequest)?;

                cache.insert(ip.clone(), weather_data.clone());
                info!("Weather data for IP {} cached successfully.", ip);

                Ok(GetWeatherResponse::Ok(Json(weather_data)))
            }
            StatusCode::BAD_REQUEST => {
                warn!("Bad request for weather data with IP: {}", ip);
                Err(WeatherError::BadRequest.into())
            }
            StatusCode::FORBIDDEN => {
                warn!("Forbidden access to weather API for IP: {}", ip);
                Err(WeatherError::Forbidden.into())
            }
            _ => {
                error!(
                    "Unexpected error occurred while fetching weather data for IP: {}",
                    ip
                );
                Err(WeatherError::Unknown.into())
            }
        }
    }
}

fn extract_ip(remote_addr: &RemoteAddr) -> Result<String, WeatherError> {
    let addr = remote_addr.to_string();
    let ip_port = addr.strip_prefix("socket://").ok_or_else(|| {
        WeatherError::IpExtraction("Address does not start with 'socket://'".to_string())
    })?;
    let (ip, _port) = ip_port.split_once(':').ok_or_else(|| {
        WeatherError::IpExtraction("Invalid address format: missing port".to_string())
    })?;
    Ok(ip.to_owned())
}

