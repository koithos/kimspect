use k8s_openapi::api::core::v1::{Container, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kimspect::{extract_registry, process_pod, split_image};

fn create_test_pod(name: &str, namespace: &str, containers: Vec<Container>) -> Pod {
    Pod {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(PodSpec {
            containers,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn create_test_container(name: &str, image: &str) -> Container {
    Container {
        name: name.to_string(),
        image: Some(image.to_string()),
        ..Default::default()
    }
}

#[test]
fn test_extract_registry() {
    let test_cases = vec![
        ("nginx:latest", "docker.io"),                         // Simple image
        ("docker.io/library/nginx:latest", "docker.io"),       // Full Docker Hub path
        ("gcr.io/google-containers/nginx:latest", "gcr.io"),   // GCR
        ("quay.io/coreos/etcd:v3.3.0", "quay.io"),             // Quay
        ("my-registry:5000/nginx:latest", "my-registry:5000"), // Private registry with port
        ("localhost:5000/nginx:latest", "localhost:5000"),     // Localhost with port
        ("127.0.0.1:5000/nginx:latest", "127.0.0.1:5000"),     // Localhost IP with port
        ("nginx", "docker.io"),                                // Bare image name
        ("", "docker.io"),                                     // Empty string
        ("invalid/registry", "docker.io"),                     // Invalid registry
        (
            "registry.example.com:8080/image:tag",
            "registry.example.com:8080",
        ), // Domain with port
        ("registry.example.com/image:tag", "registry.example.com"), // Domain without port
        ("registry.example.com/image", "registry.example.com"), // Domain without tag
        (
            "registry.example.com/image/subpath:tag",
            "registry.example.com",
        ), // Domain with subpath
        ("192.168.1.100:5000/image:latest", "192.168.1.100:5000"), // IP address registry
        ("0.0.0.0:5000/image:latest", "0.0.0.0:5000"),         // Zero IP address
    ];

    for (image, expected) in test_cases {
        assert_eq!(
            extract_registry(image),
            expected,
            "Failed for image: {}",
            image
        );
    }
}

#[test]
fn test_split_image() {
    let test_cases = vec![
        ("nginx:latest", "nginx", "latest"),
        ("nginx:1.21", "nginx", "1.21"),
        (
            "gcr.io/google-containers/nginx:v1.21",
            "gcr.io/google-containers/nginx",
            "v1.21",
        ),
        ("nginx", "nginx", "latest"),
        (
            "my-registry:5000/nginx:1.21",
            "my-registry:5000/nginx",
            "1.21",
        ),
        ("", "", "latest"),
        (
            "registry.example.com:8080/image:tag",
            "registry.example.com:8080/image",
            "tag",
        ),
        (
            "registry.example.com/image:tag",
            "registry.example.com/image",
            "tag",
        ),
        (
            "registry.example.com/image",
            "registry.example.com/image",
            "latest",
        ),
        (
            "registry.example.com/image/subpath:tag",
            "registry.example.com/image/subpath",
            "tag",
        ),
        ("image:tag@sha256:abc123", "image", "tag@sha256:abc123"),
        ("image@sha256:abc123", "image", "latest@sha256:abc123"),
        (
            "my-registry:5000/nginx:latest",
            "my-registry:5000/nginx",
            "latest",
        ),
        (
            "docker.io/library/nginx:latest",
            "docker.io/library/nginx",
            "latest",
        ),
    ];

    for (image, expected_name, expected_version) in test_cases {
        let (name, version) = split_image(image);
        assert_eq!(name, expected_name);
        assert_eq!(version, expected_version);
    }
}

// docker pull kubeflownotebookswg/base:v1.10.0-rc.1@sha256:asdasd

#[test]
fn test_process_pod() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![
            create_test_container("nginx", "nginx:latest"),
            create_test_container("redis", "redis:6.2"),
        ],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 2);

    // Check first container
    assert_eq!(images[0].pod_name, "test-pod");
    assert_eq!(images[0].namespace, "default");
    assert_eq!(images[0].container_name, "nginx");
    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "latest");
    assert_eq!(images[0].registry, "docker.io");

    // Check second container
    assert_eq!(images[1].pod_name, "test-pod");
    assert_eq!(images[1].namespace, "default");
    assert_eq!(images[1].container_name, "redis");
    assert_eq!(images[1].image_name, "redis");
    assert_eq!(images[1].image_version, "6.2");
    assert_eq!(images[1].registry, "docker.io");
}

#[test]
fn test_process_pod_with_no_spec() {
    let pod = Pod {
        metadata: ObjectMeta {
            name: Some("test-pod".to_string()),
            namespace: Some("default".to_string()),
            ..Default::default()
        },
        spec: None,
        ..Default::default()
    };

    let images = process_pod(&pod);
    assert_eq!(images.len(), 0);
}

#[test]
fn test_process_pod_with_no_containers() {
    let pod = create_test_pod("test-pod", "default", vec![]);
    let images = process_pod(&pod);
    assert_eq!(images.len(), 0);
}

#[test]
fn test_process_pod_with_no_image() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![Container {
            name: "nginx".to_string(),
            image: None,
            ..Default::default()
        }],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 0);
}

#[test]
fn test_process_pod_with_complex_image() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container("etcd", "quay.io/coreos/etcd:v3.3.0")],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].pod_name, "test-pod");
    assert_eq!(images[0].namespace, "default");
    assert_eq!(images[0].container_name, "etcd");
    assert_eq!(images[0].image_name, "coreos/etcd");
    assert_eq!(images[0].image_version, "v3.3.0");
    assert_eq!(images[0].registry, "quay.io");
}

#[test]
fn test_process_pod_with_digest() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container("nginx", "nginx@sha256:abc123def456")],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "latest@sha256:abc123def456");
    assert_eq!(images[0].registry, "docker.io");
}

#[test]
fn test_process_pod_with_tag_and_digest() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container(
            "nginx",
            "nginx:1.21@sha256:abc123def456",
        )],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "1.21@sha256:abc123def456");
    assert_eq!(images[0].registry, "docker.io");
}

#[test]
fn test_process_pod_with_registry_and_digest() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container(
            "nginx",
            "gcr.io/nginx@sha256:abc123def456",
        )],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "latest@sha256:abc123def456");
    assert_eq!(images[0].registry, "gcr.io");
}

#[test]
fn test_process_pod_with_full_path_and_digest() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container(
            "nginx",
            "gcr.io/project/nginx:1.21@sha256:abc123def456",
        )],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].image_name, "project/nginx");
    assert_eq!(images[0].image_version, "1.21@sha256:abc123def456");
    assert_eq!(images[0].registry, "gcr.io");
}

#[test]
fn test_process_pod_with_multiple_digest_containers() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![
            create_test_container("nginx", "nginx:1.21@sha256:abc123def456"),
            create_test_container("redis", "redis:6.2@sha256:def456ghi789"),
            create_test_container("etcd", "quay.io/coreos/etcd@sha256:ghi789jkl012"),
        ],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 3);

    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "1.21@sha256:abc123def456");
    assert_eq!(images[0].registry, "docker.io");

    assert_eq!(images[1].image_name, "redis");
    assert_eq!(images[1].image_version, "6.2@sha256:def456ghi789");
    assert_eq!(images[1].registry, "docker.io");

    assert_eq!(images[2].image_name, "coreos/etcd");
    assert_eq!(images[2].image_version, "latest@sha256:ghi789jkl012");
    assert_eq!(images[2].registry, "quay.io");
}

#[test]
fn test_process_pod_with_private_registry() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![create_test_container(
            "nginx",
            "my-registry:5000/nginx:1.21",
        )],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 1);

    assert_eq!(images[0].pod_name, "test-pod");
    assert_eq!(images[0].namespace, "default");
    assert_eq!(images[0].container_name, "nginx");
    assert_eq!(images[0].image_name, "nginx");
    assert_eq!(images[0].image_version, "1.21");
    assert_eq!(images[0].registry, "my-registry:5000");
}

#[test]
fn test_process_pod_with_multiple_registries() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![
            create_test_container("nginx", "docker.io/nginx:latest"),
            create_test_container("redis", "gcr.io/google-containers/redis:6.2"),
            create_test_container("etcd", "quay.io/coreos/etcd:v3.3.0"),
            create_test_container("api", "my-registry:5000/api:v1.0"),
        ],
    );

    let images = process_pod(&pod);
    assert_eq!(images.len(), 4);

    // Check each container's registry
    assert_eq!(images[0].registry, "docker.io");
    assert_eq!(images[1].registry, "gcr.io");
    assert_eq!(images[2].registry, "quay.io");
    assert_eq!(images[3].registry, "my-registry:5000");
}

#[test]
fn test_registry_filtering() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![
            create_test_container("nginx", "docker.io/nginx:latest"),
            create_test_container("redis", "gcr.io/google-containers/redis:6.2"),
            create_test_container("etcd", "quay.io/coreos/etcd:v3.3.0"),
            create_test_container("api", "my-registry:5000/api:v1.0"),
        ],
    );

    let images = process_pod(&pod);

    // Test filtering for docker.io registry
    let docker_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "docker.io")
        .collect();
    assert_eq!(docker_images.len(), 1);
    assert_eq!(docker_images[0].container_name, "nginx");
    assert_eq!(docker_images[0].image_name, "nginx");

    // Test filtering for gcr.io registry
    let gcr_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "gcr.io")
        .collect();
    assert_eq!(gcr_images.len(), 1);
    assert_eq!(gcr_images[0].container_name, "redis");
    assert_eq!(gcr_images[0].image_name, "google-containers/redis");

    // Test filtering for quay.io registry
    let quay_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "quay.io")
        .collect();
    assert_eq!(quay_images.len(), 1);
    assert_eq!(quay_images[0].container_name, "etcd");
    assert_eq!(quay_images[0].image_name, "coreos/etcd");

    // Test filtering for private registry
    let private_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "my-registry:5000")
        .collect();
    assert_eq!(private_images.len(), 1);
    assert_eq!(private_images[0].container_name, "api");
    assert_eq!(private_images[0].image_name, "api");
}

#[test]
fn test_registry_filtering_with_no_matches() {
    let pod = create_test_pod(
        "test-pod",
        "default",
        vec![
            create_test_container("nginx", "docker.io/nginx:latest"),
            create_test_container("redis", "gcr.io/google-containers/redis:6.2"),
        ],
    );

    let images = process_pod(&pod);

    // Test filtering for non-existent registry
    let non_existent_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "non-existent-registry")
        .collect();
    assert_eq!(non_existent_images.len(), 0);
}

#[test]
fn test_registry_filtering_with_empty_pod() {
    let pod = create_test_pod("test-pod", "default", vec![]);
    let images = process_pod(&pod);

    // Test filtering on empty pod
    let filtered_images: Vec<_> = images
        .iter()
        .filter(|img| img.registry == "docker.io")
        .collect();
    assert_eq!(filtered_images.len(), 0);
}
