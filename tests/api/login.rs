use crate::{spawn, LoginResponse};

#[tokio::test]
async fn test_login_api_with_valid_credentials() {
    let app = spawn().await;

    let valid_user = serde_json::json!({
        "name": "ValidUser",
        "email": "validuser@example.com",
        "password": "ValidPass123"
    });

    let response = app
        .client
        .post(&format!("{}/api/register", &app.address))
        .header("Content-Type", "application/json")
        .json(&valid_user)
        .send()
        .await
        .expect("Failed to execute register request");

    assert_eq!(response.status().as_u16(), 201);

    let login_request = serde_json::json!({
        "email": "validuser@example.com",
        "password": "ValidPass123"
    });

    let login_response = app
        .client
        .post(&format!("{}/api/login", &app.address))
        .header("Content-Type", "application/json")
        .json(&login_request)
        .send()
        .await
        .expect("Failed to execute login request");

    assert_eq!(login_response.status().as_u16(), 200);

    let login_data: LoginResponse = login_response.json().await.expect("Invalid JSON response");
    assert!(!login_data.token.is_empty(), "Token should not be empty");

    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_login_api_with_invalid_password() {
    let app = spawn().await;

    let valid_user = serde_json::json!({
        "name": "ValidUser",
        "email": "validuser@example.com",
        "password": "ValidPass123"
    });

    let response = app
        .client
        .post(&format!("{}/api/register", &app.address))
        .header("Content-Type", "application/json")
        .json(&valid_user)
        .send()
        .await
        .expect("Failed to execute register request");

    assert_eq!(response.status().as_u16(), 201);

    let invalid_login_request = serde_json::json!({
        "email": "validuser@example.com",
        "password": "WrongPass123"
    });

    let login_response = app
        .client
        .post(&format!("{}/api/login", &app.address))
        .header("Content-Type", "application/json")
        .json(&invalid_login_request)
        .send()
        .await
        .expect("Failed to execute login request");

    assert_eq!(login_response.status().as_u16(), 401);

    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_login_api_with_non_existent_user() {
    let app = spawn().await;

    let non_existent_login_request = serde_json::json!({
        "email": "nonexistentuser@example.com",
        "password": "SomePass123"
    });

    let login_response = app
        .client
        .post(&format!("{}/api/login", &app.address))
        .header("Content-Type", "application/json")
        .json(&non_existent_login_request)
        .send()
        .await
        .expect("Failed to execute login request");

    assert_eq!(login_response.status().as_u16(), 404);
    let _ = app.cleanup().await.expect("Failed to clean resources.");
}
