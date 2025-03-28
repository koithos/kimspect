use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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
        #[arg(short, long, default_value = "default")]
        namespace: String,
        /// Node name to filter pods (optional)
        #[arg(short, long)]
        node: Option<String>,
    },
}
