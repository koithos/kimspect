use anyhow::Result;
use colored::*;
use kelper::{
    cli::{Args, Commands, GetResource},
    k8s::{display_pod_images, K8sClient},
};

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser;
    let args = Args::parse();

    // Try to create the client first with better error handling
    let client = match K8sClient::new().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("\n{} {}", "Kubernetes Error:".red().bold(), e);

            // Provide helpful troubleshooting tips based on the error
            if e.to_string().contains("No kubeconfig found") {
                eprintln!("\n{}", "Troubleshooting Tips:".yellow().bold());
                eprintln!(" - Ensure kubectl is configured on your machine");
                eprintln!(" - Run 'kubectl config view' to check your configuration");
                eprintln!(" - Set KUBECONFIG environment variable if using a non-default config");
            } else if e.to_string().contains("not accessible") {
                eprintln!("\n{}", "Troubleshooting Tips:".yellow().bold());
                eprintln!(" - Check if your cluster is running with 'kubectl cluster-info'");
                eprintln!(" - Verify your network connection to the cluster");
                eprintln!(" - Check if your credentials are valid and not expired");
                eprintln!(" - Ensure your VPN is connected if accessing a remote cluster");
            }

            std::process::exit(1);
        }
    };

    match args.command {
        Commands::Get { resource } => match resource {
            GetResource::Images {
                namespace,
                node,
                pod,
            } => {
                match client
                    .get_pod_images(&namespace, node.as_deref(), pod.as_deref())
                    .await
                {
                    Ok(pod_images) => {
                        if pod_images.is_empty() {
                            println!(
                                "\n{}",
                                "No pod images found matching your criteria.".yellow()
                            );
                        } else {
                            display_pod_images(&pod_images, node.is_some());
                        }
                    }
                    Err(e) => {
                        eprintln!("\n{} {}", "Error retrieving pod images:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }
        },
    }

    Ok(())
}
