use crate::cli::formats::LogFormat;
use crate::cli::Commands;
use clap::Parser;
use std::path::PathBuf;

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
