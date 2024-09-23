use wheather_app::{configuration::Settings, start::Application};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let conf = Settings::new().expect("Failed to read configuration.");
    let application = Application::build(conf)
        .await
        .expect("Failed to build application");
    application.run_until_stopped().await?;
    Ok(())
}
