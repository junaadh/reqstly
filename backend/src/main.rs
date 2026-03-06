use dotenvy::dotenv;
use reqstly_backend::{
    AppState, build_app, config::Settings, db, error, realtime, telemetry,
};
use std::net::SocketAddr;
use tracing::Instrument;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), error::AppError> {
    dotenv().ok();

    let settings = Settings::from_env().map_err(|err| {
        error::AppError::Internal(format!("config error: {err}"))
    })?;
    let _log_guard = telemetry::init(&settings.logging)?;

    let service_span = tracing::info_span!(
        "service.lifecycle",
        service.name = %settings.logging.service_name,
        service.version = env!("CARGO_PKG_VERSION"),
        deployment.environment = %settings.logging.environment
    );

    async move {
        tracing::info!(base_url = %settings.server.base_url, "configuration loaded");

        let db = db::create_pool(&settings.database.url).await?;
        db::run_migrations(&db).await?;

        let issuer =
            format!("{}/auth/v1", settings.supabase.url.trim_end_matches('/'));

        let state = AppState {
            db,
            jwt_secret: settings.jwt.secret,
            jwt_issuer: issuer,
            realtime_hub: realtime::RealtimeHub::new(),
            ws_allowed_origins: realtime::parse_allowed_origins(
                &settings.cors.allowed_origin,
            ),
        };

        let app = build_app(state, &settings.cors.allowed_origin)?;

        let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));
        tracing::info!(%addr, "backend listening");

        let listener =
            tokio::net::TcpListener::bind(addr).await.map_err(|err| {
                error::AppError::Internal(format!("bind failed: {err}"))
            })?;

        axum::serve(listener, app).await.map_err(|err| {
            error::AppError::Internal(format!("serve failed: {err}"))
        })
    }
    .instrument(service_span)
    .await
}
