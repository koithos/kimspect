use crate::cli::Commands;
use crate::cli::formats::LogFormat;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

/// Command line arguments for the Kimspect application
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A CLI tool to serve as swiss-army knife for your operations on Kubernetes pods and nodes",
    long_about = None
)]
pub struct Args {
    /// Path to kubeconfig file (default: ~/.kube/config)
    #[arg(long = "kubeconfig", global = true)]
    pub kubeconfig: Option<PathBuf>,

    /// Enable verbose logging. Use multiple v's for increased verbosity:
    /// -v: WARN level
    /// -vv: INFO level
    /// -vvv: DEBUG level
    /// -vvvv: TRACE level
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Log format to use (default: plain for local development)
    #[arg(
        long = "log-format",
        default_value = "plain",
        global = true,
        requires = "verbose"
    )]
    pub log_format: LogFormat,

    /// Timeout for Kubernetes API requests (e.g. 60s, 1m, 1m5s). Default: 30s
    #[arg(long = "request-timeout", global = true, default_value = "30s", value_parser = parse_duration)]
    pub request_timeout: Duration,

    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

impl Args {
    /// Get the kubeconfig path, respecting the command line argument or falling back to environment variable
    ///
    /// # Returns
    ///
    /// * `Option<PathBuf>` - The path to the kubeconfig file if specified via command line or environment variable
    pub fn get_kubeconfig_path(&self) -> Option<PathBuf> {
        self.kubeconfig
            .clone()
            .or_else(|| std::env::var("KUBECONFIG").ok().map(PathBuf::from))
    }
}

/// Parse a human-readable duration string into a [`Duration`].
///
/// Supported formats: `60s`, `1m`, `1m5s` (minutes and/or seconds).
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration must not be empty".into());
    }

    let mut remaining = s;
    let mut total_secs: u64 = 0;
    let mut parsed_any = false;

    // Optional minutes component: <digits>m
    if let Some(m_pos) = remaining.find('m') {
        let minutes_str = &remaining[..m_pos];
        let minutes: u64 = minutes_str
            .parse()
            .map_err(|_| format!("invalid minutes value in '{s}'"))?;
        total_secs += minutes * 60;
        remaining = &remaining[m_pos + 1..];
        parsed_any = true;
    }

    // Optional seconds component: <digits>s
    if let Some(s_pos) = remaining.find('s') {
        let seconds_str = &remaining[..s_pos];
        let seconds: u64 = seconds_str
            .parse()
            .map_err(|_| format!("invalid seconds value in '{s}'"))?;
        total_secs += seconds;
        remaining = &remaining[s_pos + 1..];
        parsed_any = true;
    }

    if !remaining.is_empty() {
        return Err(format!("unrecognised suffix '{remaining}' in '{s}' — expected format: 60s, 1m, 1m5s"));
    }

    if !parsed_any {
        return Err(format!("'{s}' is not a valid duration — expected format: 60s, 1m, 1m5s"));
    }

    Ok(Duration::from_secs(total_secs))
}
