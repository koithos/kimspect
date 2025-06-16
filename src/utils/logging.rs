use crate::LogFormat;
use anyhow::{Context, Result};
use tracing::Level;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

/// Initialize the structured logging system with configurable formatting
pub fn init_logging(level: Level, format: LogFormat) -> Result<()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(format!("{}", level)))
        .context("Failed to create environment filter")?
        .add_directive(
            "rustls=ERROR"
                .parse()
                .context("Failed to parse rustls directive")?,
        )
        .add_directive(
            "rustls::client=ERROR"
                .parse()
                .context("Failed to parse rustls client directive")?,
        )
        .add_directive(
            "rustls::client::tls13=ERROR"
                .parse()
                .context("Failed to parse rustls tls13 directive")?,
        );

    match format {
        LogFormat::Json => {
            let json_layer = tracing_subscriber::fmt::layer()
                .with_timer(ChronoUtc::rfc_3339())
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .json()
                .with_current_span(true)
                .with_span_list(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(json_layer)
                .init();
        }
        LogFormat::Plain => {
            let plain_layer = tracing_subscriber::fmt::layer()
                .with_timer(ChronoUtc::rfc_3339())
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(plain_layer)
                .init();
        }
    }

    Ok(())
}

/// Configure the logging level based on verbosity count
/// - 0: ERROR level (default)
/// - 1: WARN level
/// - 2: INFO level
/// - 3: DEBUG level
/// - 4+: TRACE level
pub fn configure_logging(verbose: u8) -> Level {
    match verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    }
}
