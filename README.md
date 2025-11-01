# kimspect

[![Crates.io Version](https://img.shields.io/crates/v/kimspect)](https://crates.io/crates/kimspect) [![Crates.io Downloads](https://img.shields.io/crates/d/kimspect)](https://crates.io/crates/kimspect) [![release](https://github.com/koithos/kimspect/actions/workflows/release.yml/badge.svg)](https://github.com/koithos/kimspect/actions/workflows/release.yml) [![GitHub release (latest by date)](https://img.shields.io/github/v/release/koithos/kimspect)](https://github.com/koithos/kimspect/releases/latest) [![License](https://img.shields.io/crates/l/kimspect)](https://github.com/koithos/kimspect/blob/main/LICENSE) [![Project Status: Active](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active) [![GitHub last commit](https://img.shields.io/github/last-commit/koithos/kimspect)](https://github.com/koithos/kimspect/commits/main)

**kimspect** is a kubernetes container image inspection tool that provides comprehensive visibility into container images running inside your cluster. kimspect can get image information by pod, namespace, and node. Built for performance and reliability, kimspect enables container image insights with a simple, intuitive command-line interface.

<picture>
  <img alt="A GIF produced by the VHS code above" src=".vhs/demo-normal.gif">
</picture>

<picture>
  <img alt="A GIF produced by the VHS code above" src=".vhs/demo-wide.gif">
</picture>

## Features

- [x] List images in a cluster based on different filters:
  - by namespace
  - by node
  - by pod name
  - by container image registry
- [x] Advanced logging capabilities:
  - Multiple verbosity levels (-v, -vv, -vvv, -vvvv)
  - Support for both plain and JSON log formats

## Installation

kimspect can be installed using several package managers. Choose the one that suits your environment:

### Using Cargo (Rust's Package Manager)

If you have Rust and Cargo installed, you can build and install kimspect directly from the source:

```bash
cargo install kimspect
```

### Using Homebrew (macOS)

If you are on macOS and use Homebrew, you can install kimspect via our tap:

```bash
brew tap koithos/kimspect
brew install kimspect
```

### Using Krew (kubectl Plugin Manager)

If you use `kubectl` and have Krew installed, you can install kimspect as a kubectl plugin:

```bash
kubectl krew install kimspect
```

## Usage

### Get image details with multiple filters

```bash
# List Pod Images in a Namespace
kimspect get images --namespace default

# List Pod Images on a Specific Node
kimspect get images -N node-name
# or
kimspect get images --node node-name

# Note: When using the `--node` flag, the `--namespace` parameter is ignored as it will show pods from all namespaces on the specified node.

# List Images for a Specific Pod
kimspect get images -p pod-name
# or
kimspect get images --pod pod-name

# You can combine filters to get more specific results. For example, to get images for a specific pod on a specific node:
kimspect get images -N node-name -p pod-name

# List images from all namespaces
kimspect get images --all-namespaces

# Filter images by registry
kimspect get images --registry "docker.io" --namespace kube-system

# Show all images EXCEPT from docker.io
kimspect get images --exclude-registry "docker.io" --all-namespaces

# Show all images EXCEPT from specific node
kimspect get images --exclude-registry "docker.io" -n kube-system -o wide

# Filter images by registry in a specific node
kimspect get images --registry "quay.io" --node node-name

# Filter images by registry across all namespaces
kimspect get images --registry "quay.io" --all-namespaces

# Enable verbose logging
kimspect get images -v  # WARN
kimspect get images -vv  # INFO
kimspect get images -vvv  # DEBUG
kimspect get images -vvvv  # TRACE

# Use JSON log format
kimspect get images -vvv --log-format json
```

kimspect displays information in a clean tabular format:

```
kimspect get images -o wide
POD                                NAMESPACE  CONTAINER       REGISTRY         IMAGE                          VERSION      DIGEST                                                            NODE
metrics-server-8664d5f5f7-krxm6    default    linkerd-proxy   cr.l5d.io        linkerd/proxy                  edge-25.3.3  496429c2a4a430d7acb4393d01c4d5971a8e3e385e5f47ceaac29dde009e7189  multi-node-cluster-worker
metrics-server-8664d5f5f7-krxm6    default    metrics-server  registry.k8s.io  metrics-server/metrics-server  v0.7.2       ffcb2bf004d6aa0a17d90e0247cf94f2865c8901dcab4427034c341951c239f9  multi-node-cluster-worker
ollama-model-phi-6b7b67778d-np2tx  default    linkerd-proxy   cr.l5d.io        linkerd/proxy                  edge-25.3.3  496429c2a4a430d7acb4393d01c4d5971a8e3e385e5f47ceaac29dde009e7189  multi-node-cluster-worker
ollama-model-phi-6b7b67778d-np2tx  default    server          docker.io        ollama/ollama                  latest       e2c9ab127d555aa671d06d2a48ab58a2e544bbdaf6fa93313dbb4fb8bb73867c  multi-node-cluster-worker
ollama-models-store-0              default    server          docker.io        ollama/ollama                  latest       e2c9ab127d555aa671d06d2a48ab58a2e544bbdaf6fa93313dbb4fb8bb73867c  multi-node-cluster-worker
```

## Development

### Prerequisites

- Rust 1.85 or later
- Kubernetes cluster access
- `kubectl` installed & configured with your cluster

### Building from Source

```bash
# clone kimspect project
cd kimspect
cargo build --release
```

### Testing

kimspect includes comprehensive tests covering various aspects of the codebase. The tests are organized in the `tests` directory.

To run the tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_process_pod
```

## Releasing

This project uses `cargo-release` to automate the release process, ensuring that the version in `Cargo.toml` and the Git tag are synchronized.

### Prerequisites

1.  Install `cargo-release`:

    ```bash
    cargo install cargo-release
    ```

2.  Ensure your working directory is clean (all changes committed).
3.  Make sure you are on the main branch and have pulled the latest changes.

### Steps

- Run `bash scripts/cargo_release.sh <VERSION>` to update the version in the `Cargo.toml` file and create a Git tag.
- Once that is done, push the code to main, and a release workflow will be triggered which builds multi-platform binaries and distributes them via multiple channels.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
