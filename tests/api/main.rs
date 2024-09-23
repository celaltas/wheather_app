use std::{
    error::Error,
    fs::{self, File},
};

use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;
use uuid::Uuid;
use wheather_app::{
    configuration::{DatabaseSettings, Settings},
    start::Application,
};
use wiremock::MockServer;

mod login;
mod register;
mod weather;

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

pub struct TestApp {
    pub address: String,
    pub db: Pool<Sqlite>,
    db_file_path: String,
    mock_server: MockServer,
    client: Client,
}

pub async fn spawn() -> TestApp {
    let mock_server = MockServer::start().await;

    let configuration = {
        let mut c = Settings::new().expect("Failed to read configuration.");
        c.database.name = format!("tests/{}.db", Uuid::new_v4().to_string());
        c.application.port = 3000;
        c.wheather_api.weather_api_url = mock_server.uri().to_string();
        c.wheather_api.rapidapi_key = "test-api-key".to_string();
        c.wheather_api.rapidapi_host = "weatherapi.test".to_string();
        c
    };

    let db_file_path = configuration.database.name.clone();

    let pool = prepare_database(&configuration.database)
        .await
        .expect("Failed to prepare database");

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build http client.");

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("https://localhost:{}", configuration.application.port),
        db: pool,
        db_file_path,
        mock_server,
        client,
    }
}

async fn prepare_database(config: &DatabaseSettings) -> Result<SqlitePool, Box<dyn Error>> {
    create_database_file(&config.name)?;
    let db_path = format!("sqlite://{}", config.name);
    let pool = SqlitePool::connect(&db_path).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    insert_test_user(&pool, "testuser", "testuser@gmail.com", "password123").await?;
    Ok(pool)
}

fn create_database_file(db_name: &str) -> Result<(), std::io::Error> {
    let path = Path::new(db_name);
    if !path.exists() {
        File::create(path)?;
    }
    Ok(())
}

async fn insert_test_user(
    pool: &SqlitePool,
    name: &str,
    email: &str,
    plaintext: &str,
) -> Result<(), Box<dyn Error>> {
    let hash = bcrypt::hash(plaintext, 12)?;
    sqlx::query!(
        "INSERT INTO users (name, email, password_hash) VALUES (?, ?, ?)",
        name,
        email,
        hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

impl TestApp {
    pub async fn cleanup(self) -> Result<(), Box<dyn Error>> {
        self.db.close().await;
        fs::remove_file(&self.db_file_path)?;
        Ok(())
    }
}
