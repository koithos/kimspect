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

    // Create a filter that excludes sensitive information from rustls
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(format!("{}", level)))
        .unwrap()
        .add_directive("rustls=ERROR".parse().unwrap()) // Only show errors from rustls
        .add_directive("rustls::client=ERROR".parse().unwrap()) // Specifically filter client logs
        .add_directive("rustls::client::tls13=ERROR".parse().unwrap()); // Filter TLS 1.3 specific logs

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(json_layer)
        .init();

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
