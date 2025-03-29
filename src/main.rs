use anyhow::Result;
use kelper::{
    cli::{Args, Commands, GetResource},
    k8s::{display_pod_images, K8sClient},
};

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let args = Args::parse();

    match args.command {
        Commands::Get { resource } => match resource {
            GetResource::Images { namespace, node } => {
                let client = K8sClient::new().await?;
                let pod_images = client.get_pod_images(&namespace, node.as_deref()).await?;
                display_pod_images(&pod_images, node.is_none());
            }
        },
    }

    Ok(())
}
