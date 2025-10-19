use clap::ValueEnum;
use std::fmt;

/// Logging format options for Kimspect
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum LogFormat {
    /// Plain text format, better for local development
    Plain,
    /// JSON format, better for production and machine parsing
    Json,
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::Plain => write!(f, "plain"),
            LogFormat::Json => write!(f, "json"),
        }
    }
}

/// Output format options for displaying Kubernetes resource data
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// Standard output format with essential columns
    Normal,
    /// Extended output format with additional columns
    Wide,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Normal => write!(f, "normal"),
            OutputFormat::Wide => write!(f, "wide"),
        }
    }
}

impl OutputFormat {
    /// Check if this format includes registry information
    ///
    /// # Returns
    ///
    /// * `bool` - True if the format includes registry information
    pub fn includes_registry(&self) -> bool {
        matches!(self, OutputFormat::Wide)
    }

    /// Check if this format includes digest information
    ///
    /// # Returns
    ///
    /// * `bool` - True if the format includes digest information
    pub fn includes_digest(&self) -> bool {
        matches!(self, OutputFormat::Wide)
    }

    /// Check if this format includes node information
    ///
    /// # Returns
    ///
    /// * `bool` - True if the format includes node information
    pub fn includes_node(&self) -> bool {
        matches!(self, OutputFormat::Wide)
    }
}
