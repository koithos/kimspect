use clap::Parser;
use kelper::cli::{Args, Commands, GetResource};

#[test]
fn test_cli_parse_get_images_default() {
    let args = Args::parse_from(["kelper", "get", "images"]);

    // Direct destructuring since we know the exact pattern
    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;

    assert_eq!(namespace, "default");
    assert!(node.is_none());
    assert!(pod.is_none());
    assert!(!all_namespaces);
}

#[test]
fn test_cli_parse_get_images_namespace() {
    let args = Args::parse_from(["kelper", "get", "images", "--namespace", "test-ns"]);

    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;

    assert_eq!(namespace, "test-ns");
    assert!(node.is_none());
    assert!(pod.is_none());
    assert!(!all_namespaces);
}

#[test]
fn test_cli_parse_get_images_all_namespaces() {
    let args = Args::parse_from(["kelper", "get", "images", "--all-namespaces"]);

    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;
    // namespace should still be default, but all_namespaces flag should be true
    assert_eq!(namespace, "default");
    assert!(node.is_none());
    assert!(pod.is_none());
    assert!(all_namespaces);
}

#[test]
fn test_cli_parse_get_images_all_namespaces_short() {
    // Test the short flag version (-A)
    let args = Args::parse_from(["kelper", "get", "images", "-A"]);

    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;

    assert_eq!(namespace, "default");
    assert!(node.is_none());
    assert!(pod.is_none());
    assert!(all_namespaces);
}

#[test]
fn test_cli_parse_get_images_node() {
    // Test combining node filter with all-namespaces
    let args = Args::parse_from(["kelper", "get", "images", "--node", "worker1"]);

    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;

    assert_eq!(namespace, "default");
    assert_eq!(node, Some("worker1".to_string()));
    assert!(pod.is_none());
    assert!(!all_namespaces);
}

#[test]
fn test_cli_parse_get_images_pod_and_all_namespaces() {
    // Test combining pod filter with all-namespaces
    let args = Args::parse_from([
        "kelper",
        "get",
        "images",
        "--pod",
        "nginx-pod",
        "--all-namespaces",
    ]);

    let Commands::Get { resource } = args.command;

    let GetResource::Images {
        namespace,
        node,
        pod,
        all_namespaces,
    } = resource;

    assert_eq!(namespace, "default");
    assert!(node.is_none());
    assert_eq!(pod, Some("nginx-pod".to_string()));
    assert!(all_namespaces);
}

#[test]
fn test_cli_parse_get_images_namespace_and_all_namespaces_conflict() {
    // This test verifies that clap correctly rejects the combination of
    // --namespace and --all-namespaces flags

    // Use try_parse_from which returns a Result instead of panicking
    let result = Args::try_parse_from([
        "kelper",
        "get",
        "images",
        "--namespace",
        "test-ns",
        "--all-namespaces",
    ]);

    // The test passes if the parser returns an error
    assert!(
        result.is_err(),
        "Expected parser to reject conflicting arguments"
    );
}
