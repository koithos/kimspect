use anyhow::{Context, Result};
use clap::Parser;
use kelper::{display_pod_images, logging, Args, Commands, GetImages, K8sClient};
use tracing::{debug, info, instrument, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with the specified format
    logging::init_logging(logging::configure_logging(args.verbose), args.log_format)
        .context("Failed to initialize logging")?;

    debug!("Application started with args: {:?}", args);

    // Create the client with improved error context
    let client = K8sClient::new()
        .await
        .context("Failed to create Kubernetes client")?;

    info!("Successfully connected to Kubernetes cluster");

    process_commands(args, client).await?;

    debug!("Application completed successfully");
    Ok(())
}

#[instrument(skip(client), level = "debug")]
async fn process_commands(args: Args, client: K8sClient) -> Result<()> {
    match args.command {
        Commands::Get { resource } => match resource {
            GetImages::Images {
                namespace,
                node,
                pod,
                registry,
                all_namespaces,
                output,
                ..
            } => {
                debug!(
                    namespace = %namespace,
                    node = ?node,
                    pod = ?pod,
                    registry = ?registry,
                    all_namespaces = %all_namespaces,
                    output = ?output,
                    "Processing get images command"
                );

                let pod_images = client
                    .get_pod_images(
                        &namespace,
                        node.as_deref(),
                        pod.as_deref(),
                        registry.as_deref(),
                        all_namespaces,
                    )
                    .await
                    .context("Failed to retrieve pod images")?;

                if pod_images.is_empty() {
                    warn!("No pod images found matching your criteria");
                } else {
                    debug!(output = ?output, "Displaying pod images");
                    display_pod_images(&pod_images, &output)
                        .context("Failed to display pod images")?;
                    info!(
                        count = pod_images.len(),
                        "Successfully displayed pod images"
                    );
                }
            }
        },
    }
    Ok(())
}
