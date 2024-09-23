use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Object)]
pub struct WeatherErrorResponse {
    /// The HTTP status code of the error.
    pub code: u16,
    /// A human-readable description of the error.
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Object, Clone)]
pub struct WeatherResponse {
    /// Details about the location for which the weather data is provided.
    location: Location,
    /// Current weather conditions at the specified location.
    current: Current,
}

#[derive(Deserialize, Serialize, Debug, Object, Clone)]
struct Location {
    name: String,
    country: String,
    localtime: String,
}

#[derive(Deserialize, Serialize, Debug, Object, Clone)]
struct Current {
    temp_c: f64,
    wind_kph: f64,
    wind_dir: String,
    humidity: i64,
}
