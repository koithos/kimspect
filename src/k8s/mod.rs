use anyhow::{Context, Result};
use colored::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use prettytable::{row, Table};

#[derive(Debug)]
pub struct PodImage {
    pub pod_name: String,
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

    // If there's only one part, it's a Docker Hub image
    if parts.len() == 1 {
        return "docker.io".to_string();
    }

    // Get the potential registry (first part)
    let potential_registry = parts[0];

    // Check for localhost variants
    if potential_registry == "localhost"
        || potential_registry.starts_with("127.0.0.1")
        || potential_registry == "0.0.0.0"
    {
        // Handle IP with port (e.g., 127.0.0.1:5000)
        if parts.len() > 1 && potential_registry.contains(':') {
            return potential_registry.to_string();
        }
        return potential_registry.to_string();
    }

    // Check for IP address pattern (rough check)
    if potential_registry
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ':')
        && potential_registry.split('.').count() == 4
    {
        // Handle IP with port (e.g., 192.168.1.1:5000)
        return potential_registry.to_string();
    }

    // Check for registry with port (e.g., my-registry:5000)
    if potential_registry.contains(':') {
        return potential_registry.to_string();
    }

    // Check for private/public registry domains
    if potential_registry.contains('.') {
        return potential_registry.to_string();
    }

    // Default to Docker Hub if none of the above matches
    "docker.io".to_string()
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
                    registry: extract_registry(image),
                });
            }
        }
    }

    pod_images
}

pub fn split_image(image: &str) -> (String, String) {
    let parts: Vec<&str> = image.split(':').collect();
    let (name, version) = if parts.len() > 1 {
        let image_name = parts[..parts.len() - 1].join(":");
        (image_name, parts[parts.len() - 1].to_string())
    } else {
        (image.to_string(), "latest".to_string())
    };
    (name, version)
}

pub fn display_pod_images(images: &[PodImage]) {
    println!("\n{}", "Pod Images and Registries:".green().bold());
    println!("{}", "=".repeat(80));

    let mut table = Table::new();
    table.add_row(row![
        "Pod Name",
        "Namespace",
        "Container",
        "Image Name",
        "Version",
        "Registry"
    ]);

    for image in images {
        table.add_row(row![
            image.pod_name,
            image.namespace,
            image.container_name,
            image.image_name,
            image.image_version,
            image.registry.yellow()
        ]);
    }

    table.printstd();
    println!("\n{}", "=".repeat(80));
}
