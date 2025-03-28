use anyhow::{Context, Result};
use colored::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};

#[derive(Debug)]
pub struct PodImage {
    pub pod_name: String,
    pub container_name: String,
    pub image: String,
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

    pub async fn get_pod_images(&self, namespace: &str) -> Result<Vec<PodImage>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pods_list = pods.list(&Default::default())
            .await
            .context("Failed to list pods")?;

        let mut all_images = Vec::new();
        for pod in pods_list {
            all_images.extend(process_pod(&pod));
        }

        Ok(all_images)
    }
}

fn extract_registry(image: &str) -> String {
    if let Some(registry) = image.split('/').next() {
        if registry.contains('.') || registry.contains(':') {
            registry.to_string()
        } else {
            "docker.io".to_string()
        }
    } else {
        "docker.io".to_string()
    }
}

fn process_pod(pod: &Pod) -> Vec<PodImage> {
    let mut pod_images = Vec::new();
    let pod_name = pod.metadata.name.clone().unwrap_or_default();
    
    if let Some(spec) = &pod.spec {
        let containers = &spec.containers;
            for container in containers {
                if let Some(image) = &container.image {
                    pod_images.push(PodImage {
                        pod_name: pod_name.clone(),
                        container_name: container.name.clone(),
                        image: image.clone(),
                        registry: extract_registry(image),
                    });
                }
            }
        }
    
    pod_images
}

pub fn display_pod_images(images: &[PodImage]) {
    println!("\n{}", "Pod Images and Registries:".green().bold());
    println!("{}", "=".repeat(50));

    for image in images {
        println!("\n{}", "Pod:".cyan().bold());
        println!("  Name: {}", image.pod_name);
        println!("  Container: {}", image.container_name);
        println!("  Image: {}", image.image);
        println!("  Registry: {}", image.registry.yellow());
        println!("{}", "-".repeat(50));
    }
} 