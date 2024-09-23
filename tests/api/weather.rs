use std::time::{Duration, Instant};
use serde_json::json;
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, ResponseTemplate,
};

use crate::{spawn, LoginResponse};

#[tokio::test]
async fn test_weather_api_unauthorized() {
    let app = spawn().await;

    let response = app
        .client
        .get(&format!("{}/api/weather", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_client_error());
    assert_eq!(response.status(), 401);
    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_weather_api_with_auth() {
    let app = spawn().await;

    let login_request_body = json!({
        "email": "testuser@gmail.com",
        "password": "password123"
    });

    let login_response = app
        .client
        .post(&format!("{}/api/login", &app.address))
        .json(&login_request_body)
        .send()
        .await
        .expect("Failed to execute login request.");

    assert!(login_response.status().is_success(), "Login failed.");

    let login_response_json = login_response
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login response.");

    let token = login_response_json.token;

    let expected_response = json!({
        "location": {
            "name": "Some City",
            "country": "Some Country",
            "localtime": "2023-09-21 12:00"
        },
        "current": {
            "temp_c": 23.0,
            "wind_kph": 10.5,
            "wind_dir": "NE",
            "humidity": 60
        }
    });

    let weather_mock_response = ResponseTemplate::new(200).set_body_json(expected_response);

    Mock::given(method("GET"))
        .and(path("/current.json"))
        .and(query_param("q", "127.0.0.1"))
        .and(header("x-rapidapi-key", "test-api-key"))
        .and(header("x-rapidapi-host", "weatherapi.test"))
        .respond_with(weather_mock_response)
        .expect(1)
        .mount(&app.mock_server)
        .await;

    let weather_response = app
        .client
        .get(&format!("{}/api/weather", &app.address))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to execute weather request.");

    assert!(
        weather_response.status().is_success(),
        "Weather request failed."
    );

    let second_weather_response = app
        .client
        .get(&format!("{}/api/weather", &app.address))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to execute second weather request.");

    let weather_mock_response_2 = ResponseTemplate::new(200);

    Mock::given(method("GET"))
        .and(path("/current.json"))
        .and(query_param("q", "127.0.0.1"))
        .and(header("x-rapidapi-key", "test-api-key"))
        .and(header("x-rapidapi-host", "weatherapi.test"))
        .respond_with(weather_mock_response_2)
        .expect(0)
        .mount(&app.mock_server)
        .await;

    assert!(
        second_weather_response.status().is_success(),
        "Second weather request failed."
    );

    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_weather_api_rate_limit() {
    let app = spawn().await;

    for _ in 0..10 {
        let _response = app
            .client
            .get(&format!("{}/api/weather", &app.address))
            .send()
            .await
            .expect("Failed to execute request.");
    }
    let start_time = Instant::now();

    let _response = app
        .client
        .get(&format!("{}/api/weather", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    let elapsed_time = start_time.elapsed();

    assert!(elapsed_time > Duration::from_secs(50));
    let _ = app.cleanup().await.expect("Failed to clean resources.");
}
