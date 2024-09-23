use crate::spawn;

#[tokio::test]
async fn test_register_api_400_for_invalid_data() {
    let app = spawn().await;

    let test_cases = vec![
        // Invalid email format
        serde_json::json!({
            "name": "test",
            "email": "invalid_email_format",
            "password": "ValidPass123"
        }),
        // Name too short
        serde_json::json!({
            "name": "ab",
            "email": "test@example.com",
            "password": "ValidPass123"
        }),
        // Name too long
        serde_json::json!({
            "name": "a".repeat(51),
            "email": "test@example.com",
            "password": "ValidPass123"
        }),
        // Password too short
        serde_json::json!({
            "name": "ValidName",
            "email": "test@example.com",
            "password": "short"
        }),
        // Password too long
        serde_json::json!({
            "name": "ValidName",
            "email": "test@example.com",
            "password": "a".repeat(129)
        }),
    ];

    for invalid_body in test_cases {
        let response = app
            .client
            .post(&format!("{}/api/register", &app.address))
            .header("Content-Type", "application/json")
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request post request to subscriptions endpoint.");

        assert_eq!(400, response.status().as_u16(),);
    }
    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_register_api_with_valid_values() {
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
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 201);

    let rows: Vec<(String, String)> =
        sqlx::query_as::<_, (String, String)>("SELECT name, email FROM users WHERE email = ?")
            .bind("validuser@example.com")
            .fetch_all(&app.db)
            .await
            .expect("Failed to fetch user from the database");

    assert_eq!(rows.len(), 1, "User not found in the database");
    let (name, email) = &rows[0];
    assert_eq!(name, "ValidUser");
    assert_eq!(email, "validuser@example.com");

    let _ = app.cleanup().await.expect("Failed to clean resources.");
}

#[tokio::test]
async fn test_register_api_with_duplicate_values() {
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
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 201);

    let rows: Vec<(String, String)> =
        sqlx::query_as::<_, (String, String)>("SELECT name, email FROM users WHERE email = ?")
            .bind("validuser@example.com")
            .fetch_all(&app.db)
            .await
            .expect("Failed to fetch user from the database");

    assert_eq!(rows.len(), 1, "User not found in the database");
    let (name, email) = &rows[0];
    assert_eq!(name, "ValidUser");
    assert_eq!(email, "validuser@example.com");

    let duplicate_response = app
        .client
        .post(&format!("{}/api/register", &app.address))
        .header("Content-Type", "application/json")
        .json(&valid_user)
        .send()
        .await
        .expect("Failed to execute duplicate request");

    assert_eq!(
        duplicate_response.status().as_u16(),
        409,
        "Expected 409 Conflict on duplicate registration"
    );

    let _ = app.cleanup().await.expect("Failed to clean resources.");
}
