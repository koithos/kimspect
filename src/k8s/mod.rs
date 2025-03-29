use anyhow::{Context, Result};
use colored::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use prettytable::{row, Table};

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
        let client = Client::try_default()
            .await
            .context("Failed to create kube client")?;
        Ok(Self { client })
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
    ) -> Result<Vec<PodImage>> {
        let pods: Api<Pod> = if node_name.is_some() {
            Api::all(self.client.clone())
        } else {
            Api::namespaced(self.client.clone(), namespace)
        };

        let pods_list = pods
            .list(&Default::default())
            .await
            .context("Failed to list pods")?;

        let mut all_images = Vec::new();
        for pod in pods_list {
            if let Some(node) = node_name {
                if let Some(pod_node) = pod.spec.as_ref().and_then(|s| s.node_name.as_deref()) {
                    if pod_node != node {
                        continue;
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
                return (name.to_string(), format!("{}@{}", tag, &digest[1..]));
            } else {
                // This colon is part of the registry address, no tag specified
                return (
                    image_with_tag.to_string(),
                    format!("latest@{}", &digest[1..]),
                );
            }
        } else {
            // No tag present, use "latest" with the digest
            return (
                image_with_tag.to_string(),
                format!("latest@{}", &digest[1..]),
            );
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
        return (image.to_string(), "latest".to_string());
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

pub fn display_pod_images(images: &[PodImage], show_node: bool) {
    println!("\n{}", "Pod Images and Registries:".green().bold());
    println!("{}", "=".repeat(80));

    let mut table = Table::new();
    let headers = if show_node {
        row![
            "Pod Name",
            "Node",
            "Namespace",
            "Container",
            "Image Name",
            "Version",
            "Registry"
        ]
    } else {
        row![
            "Pod Name",
            "Namespace",
            "Container",
            "Image Name",
            "Version",
            "Registry"
        ]
    };
    table.add_row(headers);

    for image in images {
        let row = if show_node {
            row![
                image.pod_name,
                image.node_name.as_str(),
                image.namespace,
                image.container_name,
                image.image_name,
                image.image_version,
                image.registry.yellow()
            ]
        } else {
            row![
                image.pod_name,
                image.namespace,
                image.container_name,
                image.image_name,
                image.image_version,
                image.registry.yellow()
            ]
        };
        table.add_row(row);
    }

    table.printstd();
    println!("\n{}", "=".repeat(80));
}
