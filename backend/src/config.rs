use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub auth: AuthSettings,
    pub cors: CorsSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    pub ws_token_secret: String,
    pub ws_token_issuer: String,
    pub session_cookie_name: String,
    pub session_idle_minutes: i64,
    pub session_secure: bool,
    pub webauthn_rp_id: String,
    pub webauthn_rp_origin: String,
    pub webauthn_rp_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsSettings {
    pub allowed_origin: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
    pub format: LogFormat,
    pub service_name: String,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("server.port", 3000)?
            .set_default("server.base_url", "http://localhost:3000")?
            .set_default(
                "database.url",
                "postgres://postgres:postgres@localhost:5432/postgres",
            )?
            .set_default("auth.ws_token_secret", "dev-ws-token-secret")?
            .set_default("auth.ws_token_issuer", "reqstly.local/ws")?
            .set_default("auth.session_cookie_name", "reqstly_session")?
            .set_default("auth.session_idle_minutes", 480)?
            .set_default("auth.session_secure", false)?
            .set_default("auth.webauthn_rp_id", "localhost")?
            .set_default("auth.webauthn_rp_origin", "https://localhost")?
            .set_default("auth.webauthn_rp_name", "Reqstly")?
            .set_default("cors.allowed_origin", "https://localhost")?
            .set_default(
                "logging.level",
                "reqstly_backend=info,tower_http=info,axum=info",
            )?
            .set_default("logging.format", "json")?
            .set_default("logging.service_name", "reqstly-backend")?
            .set_default("logging.environment", "dev")?
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
