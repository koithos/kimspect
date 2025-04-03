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
    let _images = client.get_pod_images("default", None, None).await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_node() -> Result<()> {
    let client = K8sClient::new().await?;
    let _images = client
        .get_pod_images("default", Some("node-name"), None)
        .await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}
