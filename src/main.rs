mod cli;
mod k8s;

use anyhow::Result;
use cli::{Args, Commands, GetResource};
use k8s::{K8sClient, display_pod_images};

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let args = Args::parse();

    match args.command {
        Commands::Get { resource } => match resource {
            GetResource::Images { namespace } => {
                let client = K8sClient::new().await?;
                let pod_images = client.get_pod_images(&namespace).await?;
                display_pod_images(&pod_images);
            }
        },
    }

    Ok(())
} 