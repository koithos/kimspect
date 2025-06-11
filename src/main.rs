use anyhow::Result;
use clap::Parser;
use kelper::{
    cli::{Args, Commands, GetImages},
    k8s::K8sClient,
    utils::{display_pod_images, logging},
};
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging with the specified format
    logging::init_logging(
        logging::configure_logging(args.verbose),
        args.get_log_format(),
    )
    .unwrap();
    debug!("Application started with args: {:?}", args);

    // Try to create the client first with better error handling
    let client = match K8sClient::new().await {
        Ok(client) => {
            info!("Successfully connected to Kubernetes cluster");
            client
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to Kubernetes cluster");

            // Provide helpful troubleshooting tips based on the error
            if e.to_string().contains("No kubeconfig found") {
                warn!("No kubeconfig found. Troubleshooting tips:");
                info!(" - Ensure kubectl is configured on your machine");
                info!(" - Run 'kubectl config view' to check your configuration");
                info!(" - Set KUBECONFIG environment variable if using a non-default config");
            } else if e.to_string().contains("not accessible") {
                warn!("Cluster not accessible. Troubleshooting tips:");
                info!(" - Check if your cluster is running with 'kubectl cluster-info'");
                info!(" - Verify your network connection to the cluster");
                info!(" - Check if your credentials are valid and not expired");
                info!(" - Ensure your VPN is connected if accessing a remote cluster");
            }

            std::process::exit(1);
        }
    };

    match args.command {
        Commands::Get { resource } => match resource {
            GetImages::Images {
                namespace,
                node,
                pod,
                registry,
                all_namespaces,
                output,
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

                match client
                    .get_pod_images(
                        &namespace,
                        node.as_deref(),
                        pod.as_deref(),
                        registry.as_deref(),
                        all_namespaces,
                    )
                    .await
                {
                    Ok(pod_images) => {
                        if pod_images.is_empty() {
                            warn!("No pod images found matching your criteria");
                        } else {
                            debug!(
                                output = ?output,
                                "Displaying pod images"
                            );

                            display_pod_images(&pod_images, &output);
                            info!(
                                count = pod_images.len(),
                                "Successfully displayed pod images"
                            );
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to retrieve pod images");
                        std::process::exit(1);
                    }
                }
            }
        },
    }

    debug!("Application completed successfully");
    Ok(())
}
