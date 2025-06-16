use anyhow::Result;
use kelper::{K8sClient, K8sError};

#[tokio::test]
async fn test_k8s_client_creation() -> Result<()> {
    let client = K8sClient::new().await?;
    assert!(client.is_accessible().await?);
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images() -> Result<()> {
    let client = K8sClient::new().await?;
    // Updated to include the new all_namespaces parameter
    let _images = client
        .get_pod_images("default", None, None, None, false)
        .await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_node() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test that we get a ResourceNotFound error when querying a non-existent node
    let result = client
        .get_pod_images("default", Some("non-existent-node"), None, None, false)
        .await;
    assert!(matches!(result, Err(e) if e.downcast_ref::<K8sError>().is_some()));
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test the new all_namespaces functionality
    let _images = client
        .get_pod_images("default", None, None, None, true)
        .await?;
    // We can't assert specific values here as they depend on the cluster state
    // but we can verify the function doesn't panic
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_node_and_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test that we get a ResourceNotFound error when querying a non-existent node across all namespaces
    let result = client
        .get_pod_images("default", Some("non-existent-node"), None, None, true)
        .await;
    assert!(matches!(result, Err(e) if e.downcast_ref::<K8sError>().is_some()));
    Ok(())
}

#[tokio::test]
async fn test_get_pod_images_with_pod_and_all_namespaces() -> Result<()> {
    let client = K8sClient::new().await?;
    // Test that we get a ResourceNotFound error when querying a non-existent pod across all namespaces
    let result = client
        .get_pod_images("default", None, Some("non-existent-pod"), None, true)
        .await;
    assert!(matches!(result, Err(e) if e.downcast_ref::<K8sError>().is_some()));
    Ok(())
}
