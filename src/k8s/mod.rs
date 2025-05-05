use anyhow::{Context, Result};
use colored::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};
use prettytable::Table;

#[derive(Debug)]
pub struct PodImage {
    pub pod_name: String,
    pub node_name: String,
    pub namespace: String,
    pub container_name: String,
    pub image_name: String,
    pub image_version: String,
    pub registry: String,
}

pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    pub async fn new() -> Result<Self> {
        // Check if KUBECONFIG environment variable is set
        if std::env::var("KUBECONFIG").is_err() {
            // Check for default kubeconfig location
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| String::from(""));
            let default_kubeconfig = format!("{}/.kube/config", home_dir);

            if !std::path::Path::new(&default_kubeconfig).exists() {
                anyhow::bail!("No kubeconfig found. Make sure you have a valid kubeconfig at ~/.kube/config or set the KUBECONFIG environment variable.");
            }
        }

        // If we get here, a kubeconfig likely exists, so we try to create the client
        let client = Client::try_default()
            .await
            .context("Failed to create kube client. Please check your kubeconfig file.")?;

        // Create the client instance
        let k8s_client = Self { client };

        // Verify cluster accessibility
        if !k8s_client.is_accessible().await? {
            anyhow::bail!("Kubernetes cluster is not accessible. Please check your connection and cluster status.");
        }

        Ok(k8s_client)
    }

    pub async fn is_accessible(&self) -> Result<bool> {
        // Try to access the API server by making a simple request
        let api: Api<Pod> = Api::namespaced(self.client.clone(), "default");

        // We're just checking if we can connect, not if there are pods
        match api.list(&Default::default()).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Use the error's Debug representation which includes all details
                let error_message = format!("{:?}", e);

                // Check if this is an API error which we can extract more details from
                if let kube::Error::Api(api_err) = &e {
                    anyhow::bail!(
                        "Kubernetes API error: {} ({})",
                        api_err.message,
                        api_err.reason
                    )
                } else {
                    anyhow::bail!("Failed to connect to Kubernetes cluster: {}", error_message)
                }
            }
        }
    }

    pub async fn is_initialized(&self) -> Result<bool> {
        // Try to list pods in the default namespace to verify client is working
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "default");
        pods.list(&Default::default())
            .await
            .map(|_| true)
            .or_else(|_| Ok(false))
    }

    pub async fn get_pod_images(
        &self,
        namespace: &str,
        node_name: Option<&str>,
        pod_name: Option<&str>,
        all_namespaces: bool,
    ) -> Result<Vec<PodImage>> {
        let mut list_params = ListParams::default();
        let mut field_selectors = Vec::new();

        // Add field selector for node name if specified
        if let Some(node) = node_name {
            field_selectors.push(format!("spec.nodeName={}", node));
        }

        // Determine the API scope (namespaced or all)
        let pods: Api<Pod> = if all_namespaces || node_name.is_some() {
            // Query across all namespaces if --all-namespaces or --node is specified
            Api::all(self.client.clone())
        } else {
            // Otherwise, use the specified namespace
            // Add field selector for pod name *only* when querying a specific namespace
            if let Some(name) = pod_name {
                // metadata.name is unique within a namespace
                field_selectors.push(format!("metadata.name={}", name));
            }
            Api::namespaced(self.client.clone(), namespace)
        };

        // Apply combined field selectors
        if !field_selectors.is_empty() {
            list_params = list_params.fields(&field_selectors.join(","));
        }

        // Fetch pods using the configured ListParams
        let pods_list = pods.list(&list_params).await.context(format!(
            "Failed to list pods with params: {:?}",
            list_params
        ))?;

        let mut all_images = Vec::new();
        for pod in pods_list {
            // Client-side filtering for pod name is only needed when querying all namespaces
            // because metadata.name field selector doesn't work cluster-wide reliably
            // or when node_name filter was the primary cluster-wide filter.
            if (all_namespaces || node_name.is_some()) && pod_name.is_some() {
                if let Some(name_filter) = pod_name {
                    if let Some(p_name) = &pod.metadata.name {
                        if p_name != name_filter {
                            continue; // Skip pod if name doesn't match when filtering cluster-wide
                        }
                    } else {
                        continue; // Skip pods without names if filtering by name
                    }
                }
            }

            all_images.extend(process_pod(&pod));
        }

        Ok(all_images)
    }
}

pub fn extract_registry(image: &str) -> String {
    // Split the image string by '/'
    let parts: Vec<&str> = image.split('/').collect();

    // If there's only one part (e.g., "ubuntu" or "nginx"), it's a Docker Hub official image
    if parts.len() == 1 {
        return "docker.io".to_string();
    }

    // If there are two parts without dots or colons in the first part (e.g., "library/ubuntu"),
    // it's likely a Docker Hub image with namespace
    if parts.len() == 2 && !parts[0].contains('.') && !parts[0].contains(':') {
        return "docker.io".to_string();
    }

    // Get the potential registry (first part)
    let potential_registry = parts[0];

    // Check for localhost variants with or without port
    if potential_registry == "localhost"
        || potential_registry.starts_with("localhost:")
        || potential_registry.starts_with("127.0.0.1")
        || potential_registry.starts_with("0.0.0.0")
    {
        return potential_registry.to_string();
    }

    // Check for IP address pattern (more comprehensive check)
    let ip_parts: Vec<&str> = potential_registry.split(':').collect();
    let ip = ip_parts[0];
    if ip.split('.').filter(|&p| !p.is_empty()).count() == 4
        && ip.split('.').all(|p| p.parse::<u8>().is_ok())
    {
        return potential_registry.to_string();
    }

    // Check for known public registries
    let known_registries = [
        "docker.io",
        "registry.hub.docker.com",
        "ghcr.io",
        "gcr.io",
        "quay.io",
        "registry.gitlab.com",
        "mcr.microsoft.com",
        "registry.k8s.io",
        "public.ecr.aws",
        "docker.pkg.github.com",
        "pkg.dev",
    ];

    for registry in &known_registries {
        if potential_registry == *registry || potential_registry.ends_with(*registry) {
            return potential_registry.to_string();
        }
    }

    // For any domain with dots (e.g., "my-registry.example.com") or with port (e.g., "registry:5000")
    if potential_registry.contains('.') || potential_registry.contains(':') {
        return potential_registry.to_string();
    }

    // Default to Docker Hub if none of the above matches
    "docker.io".to_string()
}

pub fn split_image(image: &str) -> (String, String) {
    // First check for a digest (SHA)
    if let Some(digest_index) = image.find('@') {
        // We have a digest, get the part before the digest
        let image_with_tag = &image[..digest_index];
        let digest = &image[digest_index..]; // includes the @ symbol

        // Find the last colon which separates the image name from the tag
        if let Some(tag_index) = image_with_tag.rfind(':') {
            // Check if this colon is part of a port number in the registry
            // Look for slashes to determine if this is likely a registry port
            let last_slash_index = image_with_tag.rfind('/').unwrap_or(0);

            if tag_index > last_slash_index {
                // This colon is after the last slash, so it's a tag separator
                let name = &image_with_tag[..tag_index];
                let tag = &image_with_tag[tag_index + 1..];
                (name.to_string(), format!("{}@{}", tag, &digest[1..]))
            } else {
                // This colon is part of the registry address, no tag specified
                (
                    image_with_tag.to_string(),
                    format!("latest@{}", &digest[1..]),
                )
            }
        } else {
            // No tag present, use "latest" with the digest
            (
                image_with_tag.to_string(),
                format!("latest@{}", &digest[1..]),
            )
        }
    } else {
        // No digest, handle image name and tag
        // Find the last colon which might separate the image name from the tag
        if let Some(tag_index) = image.rfind(':') {
            // Check if this colon is part of a port number in the registry
            // Look for slashes to determine if this is likely a registry port
            let last_slash_index = image.rfind('/').unwrap_or(0);

            if tag_index > last_slash_index {
                // This colon is after the last slash, so it's a tag separator
                let name = &image[..tag_index];
                let tag = &image[tag_index + 1..];
                return (name.to_string(), tag.to_string());
            }
        }

        // No valid tag separator found
        (image.to_string(), "latest".to_string())
    }
}

pub fn process_pod(pod: &Pod) -> Vec<PodImage> {
    let mut pod_images = Vec::new();
    let pod_name = pod.metadata.name.clone().unwrap_or_default();
    let namespace = pod.metadata.namespace.clone().unwrap_or_default();

    if let Some(spec) = &pod.spec {
        let containers = &spec.containers;
        for container in containers {
            if let Some(image) = &container.image {
                let (image_name, image_version) = split_image(image);
                pod_images.push(PodImage {
                    pod_name: pod_name.clone(),
                    namespace: namespace.clone(),
                    container_name: container.name.clone(),
                    image_name,
                    image_version,
                    node_name: spec.node_name.clone().unwrap_or_default(),
                    registry: extract_registry(image),
                });
            }
        }
    }

    pod_images
}

pub fn display_pod_images(
    images: &[PodImage],
    show_node: bool,
    show_namespace: bool,
    show_pod: bool,
) {
    println!("\n{}", "Pod Images and Registries:".green().bold());
    println!("{}", "=".repeat(80));

    let mut table = Table::new();
    let mut header_cells = Vec::new();

    if show_pod {
        header_cells.push("Pod Name");
    }

    if show_node {
        header_cells.push("Node");
    }

    if show_namespace {
        header_cells.push("Namespace");
    }

    header_cells.extend(vec!["Container", "Image Name", "Version", "Registry"]);

    let header_row = header_cells.into_iter().collect::<Vec<_>>();
    table.add_row(prettytable::Row::new(
        header_row.into_iter().map(prettytable::Cell::new).collect(),
    ));

    for image in images {
        let mut row = prettytable::Row::new(Vec::new());

        if show_pod {
            row.add_cell(prettytable::Cell::new(&image.pod_name));
        }
        if show_node {
            row.add_cell(prettytable::Cell::new(&image.node_name));
        }
        if show_namespace {
            row.add_cell(prettytable::Cell::new(&image.namespace));
        }

        row.add_cell(prettytable::Cell::new(&image.container_name));
        row.add_cell(prettytable::Cell::new(&image.image_name));
        row.add_cell(prettytable::Cell::new(&image.image_version));

        row.add_cell(prettytable::Cell::new(&image.registry).style_spec("Fy"));

        table.add_row(row);
    }

    table.printstd();
    println!("\n{}", "=".repeat(80));
}
