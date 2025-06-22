use clap::Parser;
use kelper::{Args, Commands, GetImages, OutputFormat};

#[test]
fn test_cli_parse_get_images_default() {
    let args = Args::parse_from(["kelper", "get", "images"]);

    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_namespace() {
    let args = Args::parse_from(["kelper", "get", "images", "--namespace", "test-ns"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "test-ns");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_all_namespaces() {
    let args = Args::parse_from(["kelper", "get", "images", "--all-namespaces"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            // namespace should still be default, but all_namespaces flag should be true
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_all_namespaces_short() {
    // Test the short flag version (-A)
    let args = Args::parse_from(["kelper", "get", "images", "-A"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_node() {
    // Test combining node filter
    let args = Args::parse_from(["kelper", "get", "images", "--node", "worker1"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert_eq!(node, Some("worker1".to_string()));
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
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
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert_eq!(pod, Some("nginx-pod".to_string()));
            assert!(registry.is_none());
            assert!(all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_wide_output() {
    // Test wide output format
    let args = Args::parse_from(["kelper", "get", "images", "-o", "wide"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Wide);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_wide_output_long() {
    // Test wide output format with long flag
    let args = Args::parse_from(["kelper", "get", "images", "--output", "wide"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Images {
            namespace,
            node,
            pod,
            registry,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(node.is_none());
            assert!(pod.is_none());
            assert!(registry.is_none());
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Wide);
        } else {
            panic!("Expected GetImages::Images variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_images_invalid_output() {
    // Test invalid output format
    let result = Args::try_parse_from(["kelper", "get", "images", "-o", "invalid"]);
    assert!(
        result.is_err(),
        "Expected parser to reject invalid output format"
    );
}

#[test]
fn test_cli_parse_get_images_namespace_and_all_namespaces_conflict() {
    // This test verifies that clap correctly rejects the combination of
    // --namespace and --all-namespaces flags for the images command
    let result = Args::try_parse_from([
        "kelper",
        "get",
        "images",
        "--namespace",
        "test-ns",
        "--all-namespaces",
    ]);
    assert!(
        result.is_err(),
        "Expected parser to reject conflicting arguments for 'get images'"
    );
}

#[test]
fn test_cli_parse_get_pods_namespace_and_all_namespaces_conflict() {
    // This test verifies that clap correctly rejects the combination of
    // -n and -A flags for the pods command
    let result = Args::try_parse_from(["kelper", "get", "pods", "-n", "test-ns", "-A"]);
    assert!(
        result.is_err(),
        "Expected parser to reject conflicting arguments (-n and -A) for 'get pods'"
    );
}

#[test]
fn test_cli_parse_get_registries_default() {
    let args = Args::parse_from(["kelper", "get", "registries"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Registries {
            namespace,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Registries variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_registries_namespace() {
    let args = Args::parse_from(["kelper", "get", "registries", "--namespace", "test-ns"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Registries {
            namespace,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "test-ns");
            assert!(!all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Registries variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_registries_all_namespaces() {
    let args = Args::parse_from(["kelper", "get", "registries", "--all-namespaces"]);
    if let Commands::Get { resource } = args.command {
        if let GetImages::Registries {
            namespace,
            all_namespaces,
            output,
            kubeconfig: _,
        } = resource
        {
            assert_eq!(namespace, "default");
            assert!(all_namespaces);
            assert_eq!(output, OutputFormat::Normal);
        } else {
            panic!("Expected GetImages::Registries variant");
        }
    } else {
        panic!("Expected Commands::Get variant");
    }
}

#[test]
fn test_cli_parse_get_registries_namespace_and_all_namespaces_conflict() {
    let result = Args::try_parse_from([
        "kelper",
        "get",
        "registries",
        "--namespace",
        "test-ns",
        "--all-namespaces",
    ]);
    assert!(
        result.is_err(),
        "Expected parser to reject conflicting arguments for 'get registries'"
    );
}
