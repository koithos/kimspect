use clap::Subcommand;

/// Logging format options
#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    /// Plain text format, better for local development
    Plain,
    /// JSON format, better for production and machine parsing
    Json,
}

/// CLI command structure
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information about Kubernetes resources
    Get {
        #[command(subcommand)]
        resource: GetResource,
    },
}

/// Resource types that can be queried
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
