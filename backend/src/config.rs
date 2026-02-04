use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub base_url: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AzureAd {
    pub client_id: String,
    pub tenant_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Passkey {
    pub rp_id: String,
    pub origin: String,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub server: Server,
    pub jwt: Jwt,
    pub azure_ad: AzureAd,
    pub passkey: Passkey,
    pub redis: Redis,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode =
            env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            .add_source(
                File::with_name(&format!("config/{}.toml", run_mode))
                    .required(false),
            )
            .add_source(File::with_name("config/default").required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(Environment::default().separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
