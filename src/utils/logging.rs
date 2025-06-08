use std::io;
use tracing::Level;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

/// Initialize the structured logging system with JSON formatting
pub fn init_logging(level: Level) -> io::Result<()> {
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

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(format!("{}", level)))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(json_layer)
        .init();

    Ok(())
}

/// Configure the logging level based on verbosity
pub fn configure_logging(verbose: bool) -> Level {
    if verbose {
        Level::DEBUG
    } else {
        Level::ERROR
    }
}
