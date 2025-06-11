use crate::cli::formats::OutputFormat;
use clap::Subcommand;

/// CLI command structure
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information about Kubernetes resources
    Get {
        #[command(subcommand)]
        resource: GetImages,
    },
}

/// Resource types that can be queried
#[derive(Subcommand, Debug)]
pub enum GetImages {
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
        #[arg(short = 'o', long = "output", default_value = "normal")]
        output: OutputFormat,
    },
}
