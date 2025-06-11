use clap::ValueEnum;

/// Logging format options
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum LogFormat {
    /// Plain text format, better for local development
    Plain,
    /// JSON format, better for production and machine parsing
    Json,
}

/// Output format options for displaying data
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Standard output format with essential columns
    Normal,
    /// Extended output format with additional columns
    Wide,
}
