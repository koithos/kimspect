use crate::utils::logging::LogFormat;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Enable verbose logging. Use multiple v's for increased verbosity:
    /// -v: WARN level
    /// -vv: INFO level
    /// -vvv: DEBUG level
    /// -vvvv: TRACE level
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Log format to use (default: plain for local development)
    #[arg(long = "log-format", default_value = "plain", value_parser = ["plain", "json"])]
    pub log_format: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information about Kubernetes resources
    Get {
        #[command(subcommand)]
        resource: GetResource,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetResource {
    /// List pod images and their registries
    Images {
        /// Kubernetes namespace to query (defaults to "default", ignored when --node is specified)
        #[arg(
            short,
            long,
            default_value = "default",
            conflicts_with = "all_namespaces"
        )]
        namespace: String,

        #[arg(short = 'N', long = "node", conflicts_with = "all_namespaces")]
        node: Option<String>,

        #[arg(short, long)]
        pod: Option<String>,

        /// Filter pods by container image registry
        #[arg(short = 'R', long = "registry")]
        registry: Option<String>,

        #[arg(short = 'A', long = "all-namespaces", conflicts_with = "namespace")]
        all_namespaces: bool,

        /// Output format (default: normal, wide: shows additional columns)
        #[arg(short = 'o', long = "output", default_value = "normal", value_parser = ["normal", "wide"])]
        output: String,
    },
}

impl Args {
    /// Get the log format based on the command line argument
    pub fn get_log_format(&self) -> LogFormat {
        match self.log_format.as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Plain,
        }
    }
}
