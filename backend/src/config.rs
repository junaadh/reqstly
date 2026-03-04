use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub supabase: SupabaseSettings,
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
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupabaseSettings {
    pub url: String,
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
            .set_default("jwt.secret", "dev-secret")?
            .set_default("supabase.url", "http://localhost:54321")?
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
