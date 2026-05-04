use crate::utils::{KNOWN_REGISTRIES, strip_registry};
use anyhow::{Context, Result};
use hyper_timeout::TimeoutConnector;
use hyper_util::{
    client::legacy::connect::{HttpConnector, proxy::SocksV5},
    rt::TokioExecutor,
};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Node;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client, Config,
    api::ListParams,
    client::ConfigExt,
};
use std::time::Duration;
use thiserror::Error;
use tower::{BoxError, ServiceBuilder};
use tracing::{debug, error, info, instrument};

/// Represents a container image running in a Kubernetes pod
#[derive(Debug, Clone)]
pub struct PodImage {
    /// Name of the pod containing the image
    pub pod_name: String,
    /// Name of the node where the pod is running
    pub node_name: String,
    /// Kubernetes namespace of the pod
    pub namespace: String,
    /// Name of the container using this image
    pub container_name: String,
    /// Name of the container image
    pub image_name: String,
    /// Version/tag of the container image
    pub image_version: String,
    /// Registry where the image is hosted
    pub registry: String,
    /// Image digest (if available)
    pub digest: String,
    /// Image size in a human readable format (if available)
    pub image_size: String,
}

/// Errors that can occur when interacting with Kubernetes
#[derive(Debug, Error)]
pub enum K8sError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    /// Connection-related errors
    #[error("Connection error: {0}")]
    ConnectionError(String),
    /// API-related errors
    #[error("API error: {0}")]
    ApiError(String),
    /// Resource not found errors
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

/// Client for interacting with Kubernetes clusters
pub struct K8sClient {
    /// The underlying Kubernetes client
    client: Client,
}

impl K8sClient {
    /// Create a new Kubernetes client
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A new K8sClient instance or an error if initialization fails
    #[instrument(skip_all)]
    pub async fn new(request_timeout: Duration) -> Result<Self> {
        debug!("Initializing Kubernetes client");

        let kubeconfig_path = Self::get_kubeconfig_path()?;
        debug!(path = %kubeconfig_path, "Using kubeconfig path");

        let mut config = Config::infer()
            .await
            .context("Failed to infer Kubernetes configuration")?;

        debug!(
            timeout_secs = request_timeout.as_secs(),
            "Applying request timeout"
        );
        apply_request_timeout(&mut config, request_timeout);

        let client = build_client(config).context("Failed to create Kubernetes client")?;

        let k8s_client = Self { client };

        // Verify cluster accessibility
        if !k8s_client.is_accessible().await? {
            return Err(
                K8sError::ConnectionError("Kubernetes cluster is not accessible".into()).into(),
            );
        }

        info!("Successfully initialized Kubernetes client");
        Ok(k8s_client)
    }

    /// Get the path to the kubeconfig file
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The path to the kubeconfig file or an error if not found
    fn get_kubeconfig_path() -> Result<String> {
        if let Ok(path) = std::env::var("KUBECONFIG") {
            info!("Using kubeconfig from KUBECONFIG environment variable");
            return Ok(path);
        }

        debug!("KUBECONFIG not set, checking default location");
        let home_dir = std::env::var("HOME").context("Failed to get HOME directory")?;
        let default_kubeconfig = format!("{}/.kube/config", home_dir);

        if !std::path::Path::new(&default_kubeconfig).exists() {
            return Err(
                K8sError::ConfigError("No kubeconfig found at default location".into()).into(),
            );
        }

        info!("Using default kubeconfig location");
        Ok(default_kubeconfig)
    }

    /// Check if the Kubernetes cluster is accessible
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if the cluster is accessible, false otherwise
    #[instrument(skip(self))]
    pub async fn is_accessible(&self) -> Result<bool> {
        debug!("Checking cluster accessibility");
        let api: Api<Pod> = Api::namespaced(self.client.clone(), "default");

        match api.list(&Default::default()).await {
            Ok(_) => {
                debug!("Successfully connected to cluster");
                Ok(true)
            }
            Err(e) => match e {
                kube::Error::Api(api_err) => {
                    error!(
                        code = api_err.code,
                        reason = %api_err.reason,
                        message = %api_err.message,
                        "Kubernetes API error occurred"
                    );
                    Err(K8sError::ApiError(format!(
                        "{} ({}, code {})",
                        api_err.message, api_err.reason, api_err.code
                    ))
                    .into())
                }
                _ => {
                    let message = format_connection_error(&e);
                    error!(error = %e, "{message}");
                    Err(K8sError::ConnectionError(message).into())
                }
            },
        }
    }

    /// Get pod images matching the specified criteria
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace to search in
    /// * `node_name` - Optional node name filter
    /// * `pod_name` - Optional pod name filter
    /// * `registry_filter` - Optional registry filter
    /// * `all_namespaces` - Whether to search in all namespaces
    ///
    /// # Returns
    ///
    /// * `Result<Vec<PodImage>>` - List of matching pod images or an error
    #[instrument(skip(self), fields(
        namespace = %namespace,
        node = ?node_name,
        pod = ?pod_name,
        registry = ?registry_filter,
        exclude_registry = ?exclude_registry_filter,
        all_namespaces = %all_namespaces
    ))]
    pub async fn get_pod_images(
        &self,
        namespace: &str,
        node_name: Option<&str>,
        pod_name: Option<&str>,
        registry_filter: Option<&str>,
        exclude_registry_filter: &[String],
        all_namespaces: bool,
    ) -> Result<Vec<PodImage>> {
        debug!(
            namespace = %namespace,
            node = ?node_name,
            pod = ?pod_name,
            registry = ?registry_filter,
            exclude_registry = ?exclude_registry_filter,
            all_namespaces = %all_namespaces,
            "Fetching pod images"
        );

        if !all_namespaces && !self.namespace_exists(namespace).await? {
            let resource = format!("Namespace {} not found", namespace);
            return Err(K8sError::ResourceNotFound(resource).into());
        }

        let list_params = Self::build_list_params(node_name, pod_name);
        let pods = self.get_pods_api(namespace, all_namespaces, node_name)?;

        let pods_list = pods
            .list(&list_params)
            .await
            .context("Failed to list pods")?;

        debug!("Found {} pods", pods_list.items.len());

        if pods_list.items.is_empty() {
            let resource = match (node_name, pod_name) {
                (Some(node), Some(pod)) => format!("pod {} on node {}", pod, node),
                (Some(node), None) => format!("pods on node {}", node),
                (None, Some(pod)) => format!("pod {}", pod),
                (None, None) => format!("pods in namespace {}", namespace),
            };
            return Err(K8sError::ResourceNotFound(resource).into());
        }

        let mut all_images = Vec::new();
        for pod in pods_list {
            if !Self::should_process_pod(&pod, all_namespaces, node_name, pod_name) {
                continue;
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
                registry = %registry_filter,
                "Filtered images by registry"
            );
        }

        if !exclude_registry_filter.is_empty() {
            let before_count = all_images.len();
            all_images.retain(|image| !exclude_registry_filter.contains(&image.registry));
            debug!(
                before = before_count,
                after = all_images.len(),
                registries = ?exclude_registry_filter,
                "Filtered images by exclude_registry"
            );
        }

        let nodes_api: Api<Node> = Api::all(self.client.clone());

        // hashmap: node_name -> { digest -> size_bytes }
        let mut node_to_digest_size: std::collections::HashMap<
            String,
            std::collections::HashMap<String, u64>,
        > = std::collections::HashMap::new();

        // Determine needed nodes (assume every image has a digest)
        let all_nodes: std::collections::HashSet<String> = all_images
            .iter()
            .filter(|pi| !pi.node_name.is_empty())
            .map(|pi| pi.node_name.clone())
            .collect();

        node_to_digest_size.reserve(all_nodes.len());

        let Ok(node_list) = nodes_api.list(&ListParams::default()).await else {
            info!("Skipping node image size enrichment due to node list failure");
            return Ok(all_images);
        };

        node_list
            .into_iter()
            .filter_map(|node| {
                let name = node.metadata.name.clone()?;
                if !all_nodes.contains(&name) {
                    return None;
                }
                let node_images = node.status?.images?;

                let digest_map: std::collections::HashMap<String, u64> = node_images
                    .into_iter()
                    .filter_map(|img| {
                        let size = img.size_bytes.unwrap_or(0) as u64;
                        img.names.and_then(|names| {
                            names.iter().find_map(|name| {
                                name.find('@')
                                    .map(|idx| (name[idx + 1..].to_string(), size))
                            })
                        })
                    })
                    .fold(
                        std::collections::HashMap::new(),
                        |mut acc, (digest, size)| {
                            acc.entry(digest)
                                .and_modify(|v| *v = (*v).max(size))
                                .or_insert(size);
                            acc
                        },
                    );

                (!digest_map.is_empty()).then_some((name, digest_map))
            })
            .for_each(|(name, digest_map)| {
                node_to_digest_size.insert(name, digest_map);
            });

        all_images
            .iter_mut()
            .filter(|img| img.image_size.is_empty() && !img.node_name.is_empty())
            .for_each(|img| {
                if let Some(size) = node_to_digest_size
                    .get(&img.node_name)
                    .and_then(|dmap| dmap.get(&img.digest))
                {
                    img.image_size = format_bytes(*size);
                }
            });

        info!(
            total_images = all_images.len(),
            "Successfully retrieved pod images"
        );
        Ok(all_images)
    }

    /// Build list parameters for pod queries
    fn build_list_params(node_name: Option<&str>, pod_name: Option<&str>) -> ListParams {
        let mut field_selectors = Vec::new();

        if let Some(node) = node_name {
            field_selectors.push(format!("spec.nodeName={}", node));
        }

        if let Some(name) = pod_name {
            field_selectors.push(format!("metadata.name={}", name));
        }

        ListParams::default().fields(&field_selectors.join(","))
    }

    /// Get the pods API for the specified namespace
    fn get_pods_api(
        &self,
        namespace: &str,
        all_namespaces: bool,
        _node_name: Option<&str>,
    ) -> Result<Api<Pod>> {
        let api = if all_namespaces {
            Api::all(self.client.clone())
        } else {
            Api::namespaced(self.client.clone(), namespace)
        };
        Ok(api)
    }

    /// Check if a pod should be processed based on filters
    fn should_process_pod(
        pod: &Pod,
        _all_namespaces: bool,
        node_name: Option<&str>,
        pod_name: Option<&str>,
    ) -> bool {
        if let Some(name) = pod_name {
            if pod.metadata.name.as_deref() != Some(name) {
                return false;
            }
        }

        if let Some(node) = node_name {
            if pod.spec.as_ref().and_then(|s| s.node_name.as_deref()) != Some(node) {
                return false;
            }
        }

        true
    }

    /// Get unique container image registries used in the cluster
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace to search in
    /// * `all_namespaces` - Whether to search in all namespaces
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>>` - List of unique registries or an error
    #[instrument(skip(self), fields(
        namespace = %namespace,
        all_namespaces = %all_namespaces
    ))]
    pub async fn get_unique_registries(
        &self,
        namespace: &str,
        all_namespaces: bool,
    ) -> Result<Vec<String>> {
        debug!(
            namespace = %namespace,
            all_namespaces = %all_namespaces,
            "Fetching unique registries from deployments"
        );

        if !all_namespaces && !self.namespace_exists(namespace).await? {
            let resource = format!("Namespace {} not found", namespace);
            return Err(K8sError::ResourceNotFound(resource).into());
        }

        let deployments_api: Api<Deployment> = if all_namespaces {
            Api::all(self.client.clone())
        } else {
            Api::namespaced(self.client.clone(), namespace)
        };

        let deployments = deployments_api
            .list(&Default::default())
            .await
            .context("Failed to list deployments")?;

        debug!("Found {} deployments", deployments.items.len());

        if deployments.items.is_empty() {
            let resource = format!("deployments in namespace {}", namespace);
            return Err(K8sError::ResourceNotFound(resource).into());
        }

        let mut registries = std::collections::HashSet::new();
        for deploy in deployments {
            if let Some(spec) = deploy.spec {
                if let Some(pod_spec) = spec.template.spec {
                    for container in pod_spec.containers {
                        if let Some(image) = container.image {
                            let registry = extract_registry(&image);
                            registries.insert(registry);
                        }
                    }
                }
            }
        }

        let mut registries_vec: Vec<String> = registries.into_iter().collect();
        registries_vec.sort();

        info!(
            total_registries = registries_vec.len(),
            "Successfully retrieved unique registries from deployments"
        );
        Ok(registries_vec)
    }

    /// Check if a namespace exists
    ///
    /// # Arguments
    ///
    /// * `namespace` - The name of the namespace to check
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if the namespace exists, false otherwise, or an error if the API call fails
    #[instrument(skip(self), fields(namespace = %namespace))]
    pub async fn namespace_exists(&self, namespace: &str) -> Result<bool> {
        debug!(namespace = %namespace, "Checking if namespace exists");
        let namespaces_api: Api<k8s_openapi::api::core::v1::Namespace> =
            Api::all(self.client.clone());
        match namespaces_api.get(namespace).await {
            Ok(_) => {
                debug!(namespace = %namespace, "Namespace found");
                Ok(true)
            }
            Err(kube::Error::Api(api_err)) if api_err.code == 404 => {
                debug!(namespace = %namespace, "Namespace not found");
                Ok(false)
            }
            Err(e) => {
                error!(namespace = %namespace, error = %e, "Failed to check namespace existence");
                Err(
                    K8sError::ApiError(format!("Failed to check namespace {}: {}", namespace, e))
                        .into(),
                )
            }
        }
    }
}

/// Extract the registry from a container image reference
///
/// # Arguments
///
/// * `image` - The container image reference
///
/// # Returns
///
/// * `String` - The registry name
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

    // Check for localhost variants (with or without port)
    if potential_registry == "localhost"
        || potential_registry.starts_with("localhost:")
        || potential_registry.starts_with("127.0.0.1")
        || potential_registry.starts_with("0.0.0.0")
        || potential_registry.starts_with("[::1]")
    {
        return potential_registry.to_string();
    }

    // Check for IPv4 address (with or without port)
    let ip_parts: Vec<&str> = potential_registry.split(':').collect();
    let ip = ip_parts[0];
    if ip.split('.').filter(|&p| !p.is_empty()).count() == 4
        && ip.split('.').all(|p| p.parse::<u8>().is_ok())
    {
        return potential_registry.to_string();
    }

    // Check for IPv6 address (with or without port)
    if potential_registry.starts_with('[') && potential_registry.contains(']') {
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

/// Split a container image reference into name and version
///
/// # Arguments
///
/// * `image` - The container image reference
///
/// # Returns
///
/// * `(String, String)` - Tuple of (image name, image version)
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

/// Extract the digest of a container from a pod
///
/// # Arguments
///
/// * `pod` - The pod containing the container
/// * `container_name` - The name of the container
///
/// # Returns
///
/// * `Option<String>` - The container digest if available
fn extract_container_digest(pod: &Pod, container_name: &str) -> Option<String> {
    let image_id = pod
        .status
        .as_ref()?
        .container_statuses
        .as_ref()?
        .iter()
        .find(|cs| cs.name == container_name)?
        .image_id
        .clone();

    // Try to find digest after '@' first (docker-pullable format)
    image_id
        .find('@')
        .map(|at| &image_id[at + 1..])
        .filter(|digest| digest.contains(':'))
        .map(|digest| digest.to_string())
        .or_else(|| {
            ["sha256:", "sha512:"]
                .iter()
                .find_map(|algo| image_id.find(algo))
                .map(|pos| image_id[pos..].to_string())
        })
}

/// Format bytes to a human-readable string (e.g., 123.4MiB)
fn format_bytes(bytes: u64) -> String {
    const UNITS: [(u64, &str); 3] = [(1_073_741_824, "GiB"), (1_048_576, "MiB"), (1024, "KiB")];

    UNITS
        .iter()
        .find(|(unit, _)| bytes >= *unit)
        .map(|(unit, suffix)| format!("{:.1}{}", bytes as f64 / *unit as f64, suffix))
        .unwrap_or_else(|| format!("{}B", bytes))
}

/// Process a pod to extract information about its container images
///
/// # Arguments
///
/// * `pod` - The pod to process
///
/// # Returns
///
/// * `Vec<PodImage>` - List of container images in the pod
pub fn process_pod(pod: &Pod) -> Vec<PodImage> {
    let mut pod_images = Vec::new();
    let pod_name = pod.metadata.name.clone().unwrap_or_default();
    let namespace = pod.metadata.namespace.clone().unwrap_or_default();
    let node_name = pod
        .spec
        .as_ref()
        .and_then(|spec| spec.node_name.clone())
        .unwrap_or_default();

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
                    node_name: node_name.clone(),
                    registry,
                    digest,
                    image_size: String::new(),
                });
            }
        }
    }

    pod_images
}

fn format_connection_error(error: &dyn std::error::Error) -> String {
    let mut parts = vec![error.to_string()];
    let mut current = error.source();

    while let Some(source) = current {
        let source_message = source.to_string();
        let is_duplicate = parts.last() == Some(&source_message)
            || parts
                .last()
                .is_some_and(|previous| previous.contains(&source_message));

        if !is_duplicate {
            parts.push(source_message);
        }
        current = source.source();
    }

    let joined = parts.join(" -> caused by: ");
    let hint = connection_error_hint(&joined);

    if let Some(hint) = hint {
        format!(
            "Failed to connect to Kubernetes cluster.\nCause: {joined}\nHint: {hint}"
        )
    } else {
        format!("Failed to connect to Kubernetes cluster.\nCause: {joined}")
    }
}

fn apply_request_timeout(config: &mut Config, request_timeout: Duration) {
    config.connect_timeout = Some(request_timeout);
    config.read_timeout = Some(request_timeout);
    config.write_timeout = Some(request_timeout);
}

fn build_client(config: Config) -> Result<Client> {
    if let Some(proxy_url) = config.proxy_url.as_ref() {
        if proxy_url.scheme_str() == Some("socks5") {
            return build_client_with_socks5_proxy(config);
        }
    }

    Client::try_from(config).context("Failed to create default Kubernetes client")
}

fn build_client_with_socks5_proxy(config: Config) -> Result<Client> {
    let default_namespace = config.default_namespace.clone();
    let connector = build_socks5_connector(&config)?;
    let connector = config
        .rustls_https_connector_with_connector(connector)
        .context("Failed to configure TLS connector for SOCKS5 proxy")?;
    let mut connector = TimeoutConnector::new(connector);
    connector.set_connect_timeout(config.connect_timeout);
    connector.set_read_timeout(config.read_timeout);
    connector.set_write_timeout(config.write_timeout);

    let service = ServiceBuilder::new()
        .layer(config.base_uri_layer())
        .option_layer(config.auth_layer()?)
        .layer(config.extra_headers_layer()?)
        .map_err(BoxError::from)
        .service(hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(connector));

    Ok(Client::new(service, default_namespace))
}

fn build_socks5_connector(
    config: &Config,
) -> Result<SocksV5<HttpConnector>> {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);

    let proxy_url = config
        .proxy_url
        .clone()
        .context("SOCKS5 proxy requested but proxy URL is missing")?;

    let (proxy_url, auth) = strip_proxy_userinfo(&proxy_url)?;
    let connector = if let Some((user, pass)) = auth {
        SocksV5::new(proxy_url, connector).with_auth(user, pass)
    } else {
        SocksV5::new(proxy_url, connector)
    };

    Ok(connector)
}

fn strip_proxy_userinfo(proxy_url: &http::Uri) -> Result<(http::Uri, Option<(String, String)>)> {
    let Some(authority) = proxy_url.authority() else {
        return Ok((proxy_url.clone(), None));
    };

    let authority_str = authority.as_str();
    let Some((userinfo, host_port)) = authority_str.rsplit_once('@') else {
        return Ok((proxy_url.clone(), None));
    };

    let (user, pass) = userinfo.split_once(':').unwrap_or((userinfo, ""));
    let mut sanitized = format!(
        "{}://{}",
        proxy_url.scheme_str().unwrap_or("socks5"),
        host_port
    );

    if let Some(path_and_query) = proxy_url.path_and_query() {
        sanitized.push_str(path_and_query.as_str());
    }

    let sanitized_proxy_url = sanitized
        .parse()
        .context("Failed to parse SOCKS5 proxy URL without userinfo")?;

    Ok((
        sanitized_proxy_url,
        Some((percent_decode(user)?, percent_decode(pass)?)),
    ))
}

fn percent_decode(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let mut decoded = String::with_capacity(value.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                anyhow::bail!("invalid percent-encoding in proxy credentials");
            }

            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .context("Invalid UTF-8 in percent-encoded proxy credential")?;
            let byte = u8::from_str_radix(hex, 16)
                .with_context(|| format!("Invalid percent-encoding '%{hex}' in proxy credential"))?;
            decoded.push(byte as char);
            index += 3;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }

    Ok(decoded)
}

fn connection_error_hint(message: &str) -> Option<&'static str> {
    let lower = message.to_ascii_lowercase();

    if lower.contains("deadline has elapsed") || lower.contains("timed out") {
        return Some(
            "the Kubernetes API connection timed out; verify network/proxy reachability and, if needed, increase --request-timeout",
        );
    }

    if lower.contains("socks error") && lower.contains("does not support user/pass authentication")
    {
        return Some(
            "the configured SOCKS5 proxy does not accept username/password authentication; verify the proxy auth method expected by this cluster context",
        );
    }

    if lower.contains("socks error") && lower.contains("credentials not accepted") {
        return Some(
            "the configured SOCKS5 proxy rejected the supplied credentials; verify the proxy username and password in your kubeconfig",
        );
    }

    if lower.contains("socks error") && lower.contains("authentication incorrectly") {
        return Some(
            "the SOCKS5 proxy and client disagreed on the negotiated authentication method; verify the proxy configuration for this context",
        );
    }

    if lower.contains("socks error") && lower.contains("authentication") {
        return Some(
            "the configured SOCKS5 proxy failed during authentication; verify the proxy auth method and credentials for this context",
        );
    }

    if lower.contains("socks error") {
        return Some("the configured SOCKS5 proxy failed the connection; verify proxy settings");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        apply_request_timeout, connection_error_hint, format_connection_error, percent_decode,
        strip_proxy_userinfo,
    };
    use kube::Config;
    use std::error::Error;
    use std::fmt;
    use std::time::Duration;

    #[derive(Debug)]
    struct TestError {
        message: &'static str,
        source: Option<Box<dyn Error + Send + Sync>>,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl Error for TestError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.source.as_deref().map(|source| source as _)
        }
    }

    #[test]
    fn test_format_connection_error_preserves_cause() {
        let message = format_connection_error(&TestError {
            message: "proxy requires kube/socks5",
            source: None,
        });
        assert_eq!(
            message,
            "Failed to connect to Kubernetes cluster.\nCause: proxy requires kube/socks5"
        );
    }

    #[test]
    fn test_format_connection_error_includes_source_chain() {
        let message = format_connection_error(&TestError {
            message: "ServiceError: client error (Connect)",
            source: Some(Box::new(TestError {
                message: "tcp connect error",
                source: Some(Box::new(TestError {
                    message: "Connection refused (os error 111)",
                    source: None,
                })),
            })),
        });

        assert_eq!(
            message,
            "Failed to connect to Kubernetes cluster.\nCause: ServiceError: client error (Connect) -> caused by: tcp connect error -> caused by: Connection refused (os error 111)"
        );
    }

    #[test]
    fn test_format_connection_error_skips_redundant_wrapped_source() {
        let message = format_connection_error(&TestError {
            message: "ServiceError: client error (Connect)",
            source: Some(Box::new(TestError {
                message: "client error (Connect)",
                source: Some(Box::new(TestError {
                    message: "SOCKS error: server does not support user/pass authentication",
                    source: None,
                })),
            })),
        });

        assert_eq!(
            message,
            "Failed to connect to Kubernetes cluster.\nCause: ServiceError: client error (Connect) -> caused by: SOCKS error: server does not support user/pass authentication\nHint: the configured SOCKS5 proxy does not accept username/password authentication; verify the proxy auth method expected by this cluster context"
        );
    }

    #[test]
    fn test_apply_request_timeout_updates_connect_read_and_write() {
        let mut config = Config::new("https://example.invalid".parse().unwrap());
        let timeout = Duration::from_secs(90);

        apply_request_timeout(&mut config, timeout);

        assert_eq!(config.connect_timeout, Some(timeout));
        assert_eq!(config.read_timeout, Some(timeout));
        assert_eq!(config.write_timeout, Some(timeout));
    }

    #[test]
    fn test_connection_error_hint_for_timeout() {
        let hint = connection_error_hint(
            "ServiceError: client error (Connect) -> caused by: deadline has elapsed",
        );
        assert_eq!(
            hint,
            Some(
                "the Kubernetes API connection timed out; verify network/proxy reachability and, if needed, increase --request-timeout"
            )
        );
    }

    #[test]
    fn test_connection_error_hint_for_socks_auth() {
        let hint = connection_error_hint(
            "ServiceError: client error (Connect) -> caused by: SOCKS error: server does not support user/pass authentication",
        );
        assert_eq!(
            hint,
            Some(
                "the configured SOCKS5 proxy does not accept username/password authentication; verify the proxy auth method expected by this cluster context"
            )
        );
    }

    #[test]
    fn test_connection_error_hint_for_socks_bad_credentials() {
        let hint = connection_error_hint(
            "ServiceError: client error (Connect) -> caused by: SOCKS error: credentials not accepted",
        );
        assert_eq!(
            hint,
            Some(
                "the configured SOCKS5 proxy rejected the supplied credentials; verify the proxy username and password in your kubeconfig"
            )
        );
    }

    #[test]
    fn test_connection_error_hint_for_socks_method_mismatch() {
        let hint = connection_error_hint(
            "ServiceError: client error (Connect) -> caused by: SOCKS error: server implements authentication incorrectly",
        );
        assert_eq!(
            hint,
            Some(
                "the SOCKS5 proxy and client disagreed on the negotiated authentication method; verify the proxy configuration for this context"
            )
        );
    }

    #[test]
    fn test_strip_proxy_userinfo_extracts_credentials() {
        let proxy_url: http::Uri = "socks5://user:pass@proxy.example.com:1080/".parse().unwrap();

        let (sanitized, auth) = strip_proxy_userinfo(&proxy_url).unwrap();

        assert_eq!(sanitized.to_string(), "socks5://proxy.example.com:1080/");
        assert_eq!(auth, Some(("user".into(), "pass".into())));
    }

    #[test]
    fn test_strip_proxy_userinfo_decodes_percent_encoding() {
        let proxy_url: http::Uri =
            "socks5://foundation-platform:abc%40123@proxy.example.com:1080/".parse().unwrap();

        let (_, auth) = strip_proxy_userinfo(&proxy_url).unwrap();

        assert_eq!(auth, Some(("foundation-platform".into(), "abc@123".into())));
    }

    #[test]
    fn test_strip_proxy_userinfo_without_auth_keeps_proxy() {
        let proxy_url: http::Uri = "socks5://proxy.example.com:1080/".parse().unwrap();

        let (sanitized, auth) = strip_proxy_userinfo(&proxy_url).unwrap();

        assert_eq!(sanitized, proxy_url);
        assert!(auth.is_none());
    }

    #[test]
    fn test_percent_decode_rejects_invalid_encoding() {
        let err = percent_decode("%zz").unwrap_err().to_string();
        assert!(err.contains("Invalid percent-encoding"));
    }
}
