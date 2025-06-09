use crate::utils::{strip_registry, KNOWN_REGISTRIES};
use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};
use tracing::{debug, error, info, instrument};

#[derive(Debug)]
pub struct PodImage {
    pub pod_name: String,
    pub node_name: String,
    pub namespace: String,
    pub container_name: String,
    pub image_name: String,
    pub image_version: String,
    pub registry: String,
    pub digest: String,
}

pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    #[instrument(skip_all)]
    pub async fn new() -> Result<Self> {
        debug!("Initializing Kubernetes client");

        // Check if KUBECONFIG environment variable is set
        if std::env::var("KUBECONFIG").is_err() {
            debug!("KUBECONFIG not set, checking default location");
            // Check for default kubeconfig location
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| String::from(""));
            let default_kubeconfig = format!("{}/.kube/config", home_dir);

            if !std::path::Path::new(&default_kubeconfig).exists() {
                error!("No kubeconfig found at default location");
                anyhow::bail!("No kubeconfig found. Make sure you have a valid kubeconfig at ~/.kube/config or set the KUBECONFIG environment variable.");
            }
            info!("Using default kubeconfig location");
        } else {
            info!("Using kubeconfig from KUBECONFIG environment variable");
        }

        // If we get here, a kubeconfig likely exists, so we try to create the client
        let client = match Client::try_default().await {
            Ok(client) => client,
            Err(e) => {
                // Handle specific error types without exposing sensitive information
                match e {
                    kube::Error::InferConfig(_) => {
                        error!("Failed to infer cluster configuration");
                        anyhow::bail!("Failed to infer cluster configuration. Please check your kubeconfig file.")
                    }
                    _ => {
                        error!("Failed to create kube client");
                        anyhow::bail!("Failed to create kube client. Please check your kubeconfig file and cluster connection.")
                    }
                }
            }
        };

        // Create the client instance
        let k8s_client = Self { client };

        // Verify cluster accessibility
        if !k8s_client.is_accessible().await? {
            error!("Failed to verify cluster accessibility");
            anyhow::bail!("Kubernetes cluster is not accessible. Please check your connection and cluster status.");
        }

        info!("Successfully initialized Kubernetes client");
        Ok(k8s_client)
    }

    #[instrument(skip(self))]
    pub async fn is_accessible(&self) -> Result<bool> {
        debug!("Checking cluster accessibility");
        // Try to access the API server by making a simple request
        let api: Api<Pod> = Api::namespaced(self.client.clone(), "default");

        // We're just checking if we can connect, not if there are pods
        match api.list(&Default::default()).await {
            Ok(_) => {
                debug!("Successfully connected to cluster");
                Ok(true)
            }
            Err(e) => {
                // Handle specific error types without exposing sensitive information
                match e {
                    kube::Error::Api(api_err) => {
                        error!("Kubernetes API error occurred");
                        anyhow::bail!(
                            "Kubernetes API error: {} ({})",
                            api_err.message,
                            api_err.reason
                        )
                    }
                    _ => {
                        error!("Failed to connect to Kubernetes cluster");
                        anyhow::bail!("Failed to connect to Kubernetes cluster. Please check your connection and cluster status.")
                    }
                }
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn is_initialized(&self) -> Result<bool> {
        // Try to list pods in the default namespace to verify client is working
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), "default");
        match pods.list(&Default::default()).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    #[instrument(skip(self), fields(
        namespace = %namespace,
        node = ?node_name,
        pod = ?pod_name,
        registry = ?registry_filter,
        all_namespaces = %all_namespaces
    ))]
    pub async fn get_pod_images(
        &self,
        namespace: &str,
        node_name: Option<&str>,
        pod_name: Option<&str>,
        registry_filter: Option<&str>,
        all_namespaces: bool,
    ) -> Result<Vec<PodImage>> {
        debug!(
            namespace = %namespace,
            node = ?node_name,
            pod = ?pod_name,
            registry = ?registry_filter,
            all_namespaces = %all_namespaces,
            "Fetching pod images"
        );

        let mut field_selectors = String::new();

        if let Some(node) = node_name {
            field_selectors.push_str(&format!("spec.nodeName={}", node));
        }

        if let Some(name) = pod_name {
            if !field_selectors.is_empty() {
                field_selectors.push(',');
            }
            field_selectors.push_str(&format!("metadata.name={}", name));
        }

        let list_params = if !field_selectors.is_empty() {
            ListParams::default().fields(&field_selectors)
        } else {
            ListParams::default()
        };

        let pods: Api<Pod> = if all_namespaces || node_name.is_some() {
            debug!("Querying pods across all namespaces");
            Api::all(self.client.clone())
        } else {
            debug!("Querying pods in namespace: {}", namespace);
            Api::namespaced(self.client.clone(), namespace)
        };

        let pods_list = match pods.list(&list_params).await {
            Ok(list) => list,
            Err(e) => {
                // Handle specific error types without exposing sensitive information
                match e {
                    kube::Error::Api(api_err) => {
                        error!("Kubernetes API error occurred");
                        anyhow::bail!(
                            "Kubernetes API error: {} ({})",
                            api_err.message,
                            api_err.reason
                        )
                    }
                    _ => {
                        error!("Failed to list pods");
                        anyhow::bail!(
                            "Failed to list pods. Please check your connection and cluster status."
                        )
                    }
                }
            }
        };

        debug!("Found {} pods", pods_list.items.len());

        let mut all_images = Vec::new();
        for pod in pods_list {
            // Perform client-side filtering for pod name when querying cluster-wide.
            if let (true, Some(name_filter)) = ((all_namespaces || node_name.is_some()), pod_name) {
                match &pod.metadata.name {
                    Some(p_name) if p_name != name_filter => {
                        debug!("Skipping pod (doesn't match filter)");
                        continue;
                    }
                    None => {
                        debug!("Skipping pod without name");
                        continue;
                    }
                    _ => {}
                }
            }

            let pod_images = process_pod(&pod);
            debug!(images = pod_images.len(), "Processed pod images");
            all_images.extend(pod_images);
        }

        if let Some(registry_filter) = registry_filter {
            let before_count = all_images.len();
            all_images.retain(|image| image.registry == registry_filter);
            debug!(
                before = before_count,
                after = all_images.len(),
                "Filtered images by registry"
            );
        }

        info!(
            total_images = all_images.len(),
            "Successfully retrieved pod images"
        );
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
    let known_registries = KNOWN_REGISTRIES;

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

fn extract_container_digest(pod: &Pod, container_name: &str) -> Option<String> {
    pod.status
        .as_ref()?
        .container_statuses
        .as_ref()?
        .iter()
        .find(|cs| cs.name == container_name)?
        .image_id
        .split(':')
        .nth(1)
        .map(String::from)
}

pub fn process_pod(pod: &Pod) -> Vec<PodImage> {
    let mut pod_images = Vec::new();
    let pod_name = pod.metadata.name.clone().unwrap_or_default();
    let namespace = pod.metadata.namespace.clone().unwrap_or_default();

    if let Some(spec) = &pod.spec {
        let containers = &spec.containers;
        for container in containers {
            if let Some(image) = &container.image {
                let registry = extract_registry(image);
                let (_image_name, image_version) = split_image(image);
                let image_name = strip_registry(&_image_name, &registry);
                let digest = extract_container_digest(pod, &container.name).unwrap_or_default();

                pod_images.push(PodImage {
                    pod_name: pod_name.clone(),
                    namespace: namespace.clone(),
                    container_name: container.name.clone(),
                    image_name,
                    image_version,
                    node_name: spec.node_name.clone().unwrap_or_default(),
                    registry,
                    digest,
                });
            }
        }
    }

    pod_images
}
