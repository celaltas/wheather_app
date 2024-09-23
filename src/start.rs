use crate::{
    configuration::Settings,
    user::{self, middleware::JWTAuth},
    weather::{self, model::WeatherResponse},
};
use poem::{
    listener::{Listener, RustlsCertificate, RustlsConfig, RustlsListener, TcpListener},
    middleware::{Cors, TowerLayerCompatExt, Tracing},
    EndpointExt, Route, Server,
};
use poem_openapi::OpenApiService;
use quick_cache::sync::Cache;
use reqwest::Client;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{error::Error, fs, future::Future, pin::Pin, sync::Arc, time::Duration};
use tower::limit::RateLimitLayer;

type PinnedServer = Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send>>;

pub struct Application {
    port: u16,
    server: PinnedServer,
}

impl Application {
    pub async fn build(conf: Settings) -> Result<Application, Box<dyn Error>> {
        let db_path = format!("sqlite://{}", conf.database.name);
        let pool = SqlitePool::connect(&db_path).await?;
        let http_client = Client::new();
        let cache = Arc::new(Cache::<String, WeatherResponse>::new(
            conf.application.item_capacity,
        ));

        sqlx::migrate!().run(&pool).await?;

        let addr = format!("{}:{}", conf.application.host, conf.application.port);
        let port = conf.application.port;

        let cert = fs::read(&conf.tls.cert_path)?;
        let key = fs::read(&conf.tls.key_path)?;

        let listener: RustlsListener<TcpListener<String>, RustlsConfig> = TcpListener::bind(addr)
            .rustls(RustlsConfig::new().fallback(RustlsCertificate::new().key(key).cert(cert)));

        let server = build_app(listener, pool, conf, cache, http_client);

        Ok(Application { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

fn build_app(
    listener: RustlsListener<TcpListener<String>, RustlsConfig>,
    pool: Pool<Sqlite>,
    conf: Settings,
    cache: Arc<Cache<String, WeatherResponse>>,
    http_client: Client,
) -> PinnedServer {
    let endpoints = (
        user::register::api::RegisterApi,
        user::login::api::LoginApi,
        weather::api::WeatherApi,
    );

    let api_service =
        OpenApiService::new(endpoints, &conf.application.name, &conf.application.version).server(
            format!(
                "https://{}:{}/api",
                conf.application.host, conf.application.port
            ),
        );

    let ui = api_service.swagger_ui();

    let cors = Cors::new()
        .allow_origins(["http://localhost:3000"])
        .allow_methods(["GET", "POST"])
        .allow_headers(["Authorization", "Content-Type"])
        .max_age(3600);

    let jwt_settings = conf.jwt.clone();
    let jwt_auth = JWTAuth::new(jwt_settings);

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs", ui)
        .with(jwt_auth)
        .with(Tracing)
        .with(RateLimitLayer::new(10, Duration::from_secs(60)).compat())
        .with(cors)
        .data(pool)
        .data(conf)
        .data(cache)
        .data(http_client);

    let server = Server::new(listener).run_with_graceful_shutdown(
        app,
        async move {
            let _ = tokio::signal::ctrl_c().await;
        },
        Some(Duration::from_secs(5)),
    );
    Box::pin(server)
}
