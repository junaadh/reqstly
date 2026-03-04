use crate::{
    config::{LogFormat, LoggingSettings},
    error::AppError,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init(logging: &LoggingSettings) -> Result<WorkerGuard, AppError> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(logging.level.as_str()))
        .map_err(|err| {
            AppError::Internal(format!("invalid log filter: {err}"))
        })?;

    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());

    let subscriber_builder = fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(false);

    match logging.format {
        LogFormat::Json => subscriber_builder
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(true)
            .init(),
        LogFormat::Pretty => subscriber_builder.pretty().init(),
        LogFormat::Compact => subscriber_builder.compact().init(),
    }

    Ok(guard)
}
