use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct AzureAd {
    pub client_id: String,
    pub tenant_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct Passkey {
    pub rp_id: String,
    pub origin: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub server: Server,
    pub jwt: Jwt,
    pub azure_ad: AzureAd,
    pub passkey: Passkey,
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            database: Database {
                url: "postgresql://reqstly:password@localhost:5432/reqstly"
                    .to_string(),
            },
            server: Server { port: 3000 },
            jwt: Jwt {
                secret: "change-this-secret-in-production".to_string(),
                expiration_hours: 24,
            },
            azure_ad: AzureAd {
                client_id: "".to_string(),
                tenant_id: "".to_string(),
                client_secret: "".to_string(),
            },
            passkey: Passkey {
                rp_id: "localhost".to_string(),
                origin: "http://localhost:5173".to_string(),
            },
        }
    }
}
