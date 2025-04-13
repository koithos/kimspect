use anyhow::Result;
use kelper::k8s::K8sClient;

#[tokio::test]
async fn test_k8s_client_creation() -> Result<()> {
    let client = K8sClient::new().await?;
    assert!(client.is_initialized().await?);
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images() -> Result<()> {
    let client = K8sClient::new().await?;
    // Updated to include the new all_namespaces parameter
    let _images = client.get_pod_images("default", None, None, false).await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_node() -> Result<()> {
    let client = K8sClient::new().await?;
    // Updated to include the new all_namespaces parameter
    let _images = client
        .get_pod_images("default", Some("node-name"), None, false)
        .await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test the new all_namespaces functionality
    let _images = client.get_pod_images("default", None, None, true).await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_node_and_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test combination of node filter and all_namespaces
    let _images = client
        .get_pod_images("default", Some("node-name"), None, true)
        .await?;
    // Verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_pod_and_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test combination of pod filter and all_namespaces
    let _images = client
        .get_pod_images("default", None, Some("pod-name"), true)
        .await?;
    // Verify the function doesn't panic
    Ok(())
}
