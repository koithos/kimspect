use crate::LogFormat;
use anyhow::{Context, Result};
use tracing::Level;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

/// Initialize the structured logging system with configurable formatting
///
/// # Arguments
///
/// * `level` - The logging level to use
/// * `format` - The format to use for log messages (JSON or plain text)
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn init_logging(level: Level, format: LogFormat) -> Result<()> {
    let filter_layer = create_filter_layer(level)?;

    match format {
        LogFormat::Json => init_json_logging(filter_layer),
        LogFormat::Plain => init_plain_logging(filter_layer),
    }

    Ok(())
}

/// Create the filter layer for logging
///
/// # Arguments
///
/// * `level` - The logging level to use
///
/// # Returns
///
/// * `Result<EnvFilter>` - The configured filter layer or error
fn create_filter_layer(level: Level) -> Result<EnvFilter> {
    Ok(EnvFilter::try_from_default_env()
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
        ))
}

/// Initialize JSON format logging
///
/// # Arguments
///
/// * `filter_layer` - The filter layer to use
fn init_json_logging(filter_layer: EnvFilter) {
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

/// Initialize plain text format logging
///
/// # Arguments
///
/// * `filter_layer` - The filter layer to use
fn init_plain_logging(filter_layer: EnvFilter) {
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

/// Configure the logging level based on verbosity count
///
/// # Arguments
///
/// * `verbose` - The verbosity level (0-4)
///
/// # Returns
///
/// * `Level` - The corresponding logging level
///
/// # Examples
///
/// ```txt
/// - 0; ERROR level (default)
/// - 1; WARN level
/// - 2; INFO level
/// - 3; DEBUG level
/// - 4+; TRACE level
/// ```
pub fn configure_logging(verbose: u8) -> Level {
    match verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    }
}
