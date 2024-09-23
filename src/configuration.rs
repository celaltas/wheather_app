use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub wheather_api: WheatherApiSettings,
    pub jwt: JwtSettings,
    pub tls: TLSSettings,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub name: String,
    pub version: String,
    pub item_capacity: usize,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct DatabaseSettings {
    pub name: String,
    pub max_connections: u32,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct WheatherApiSettings {
    pub rapidapi_key: String,
    pub rapidapi_host: String,
    pub weather_api_url: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct JwtSettings {
    pub secret: String,
    pub expire_min: i64,
}

impl JwtSettings {
    pub fn new(secret: String, expire_min: i64) -> Self {
        Self { secret, expire_min }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TLSSettings {
    pub cert_path: String,
    pub key_path: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let conf_path = base_path.join("configuration/base");
        let builder =
            Config::builder().add_source(File::new(conf_path.to_str().unwrap(), FileFormat::Yaml));
        let config = builder.build()?;
        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::{
        ApplicationSettings, DatabaseSettings, JwtSettings, Settings, TLSSettings,
        WheatherApiSettings,
    };

    #[test]
    fn test_get_configuration() {
        let res = Settings::new();
        assert!(res.is_ok());
        let expected = Settings {
            application: ApplicationSettings {
                host: "127.0.0.1".to_string(),
                port: 3000,
                debug: true,
                name: "Wheather App".to_string(),
                version: "1.0.0".to_string(),
                item_capacity: 100,
            },
            database: DatabaseSettings {
                name: "data.db".to_string(),
                max_connections: 5,
            },
            wheather_api: WheatherApiSettings {
                rapidapi_key: "rapidapi_key".to_string(),
                rapidapi_host: "rapidapi_host".to_string(),
                weather_api_url: "weather_api_url".to_string(),
            },
            jwt: JwtSettings {
                secret: "secret".to_string(),
                expire_min: 30,
            },
            tls: TLSSettings {
                cert_path: "cert_path".to_string(),
                key_path: "key_path".to_string(),
            },
        };
        let res = res.unwrap();

        assert_eq!(res.application.host, expected.application.host);
        assert_eq!(res.application.name, expected.application.name);
    }
}
