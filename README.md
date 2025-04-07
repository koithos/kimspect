# Kelper

A CLI tool to query Kubernetes pod images and their registries. Kelper helps you quickly inspect container images running in your Kubernetes clusters, with support for filtering by namespace, node, and pod name.

## Features

- List all pod images in a namespace
- Filter pod images by node name
- Filter pod images by pod name
- Display image details in a clean tabular format
- Show image names and versions separately
- Identify image registries

## Installation

Kelper can be installed using several package managers. Choose the one that suits your environment:

### Using Cargo (Rust's Package Manager)

If you have Rust and Cargo installed, you can build and install Kelper directly from the source:

```bash
cargo install kelper
```

### Using Homebrew (macOS)

If you are on macOS and use Homebrew, you can install Kelper via our tap:

```bash
brew tap aliabbasjaffri/kelper
brew install kelper
```

### Using Krew (kubectl Plugin Manager)

If you use `kubectl` and have Krew installed, you can install Kelper as a kubectl plugin:

```bash
kubectl krew install kelper
```

After installation via Krew, you can use Kelper as a kubectl command:

```bash
kubectl kelper get images --namespace default
```

## Usage

### List Pod Images in a Namespace

To list all pod images in a specific namespace:

```bash
kelper get images --namespace default
```

### List Pod Images on a Specific Node

To list all pod images running on a specific node across all namespaces:

```bash
kelper get images -N node-name
# or
kelper get images --node node-name
```

Note: When using the `--node` flag, the `--namespace` parameter is ignored as it will show pods from all namespaces on the specified node.

### List Images for a Specific Pod

To list images for a specific pod:

```bash
kelper get images -p pod-name
# or
kelper get images --pod pod-name
```

You can combine filters to get more specific results. For example, to get images for a specific pod on a specific node:

```bash
kelper get images -N node-name -p pod-name
```

## Output Format

Kelper displays information in a clean tabular format:

```
Pod Images and Registries:
================================================================================
+----------------+-----------+------------+------------+---------+-------------+
| Pod Name       | Namespace | Container  | Image Name | Version | Registry    |
+----------------+-----------+------------+------------+---------+-------------+
| nginx-pod      | default   | nginx      | nginx      | latest  | docker.io   |
| redis-pod      | prod      | redis      | redis      | 6.2     | docker.io   |
| api-pod        | staging   | api        | api        | v1.0    | registry.io |
+----------------+-----------+------------+------------+---------+-------------+
================================================================================
```

The output includes:

- Pod Name: The name of the Kubernetes pod
- Namespace: The Kubernetes namespace
- Container: The container name within the pod
- Image Name: The name of the container image
- Version: The image version/tag
- Registry: The container registry hosting the image

## Development

### Prerequisites

- Rust 1.70 or later
- Kubernetes cluster access
- `kubectl` configured with your cluster

### Building from Source

```bash
# clone kelper project
cd kelper
cargo build --release
```

### Testing

Kelper includes comprehensive tests covering various aspects of the codebase. The tests are organized in the `tests` directory and include:

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

- Run `bash scripts/cargo_release.sh <VERSION>` to update the version in the `cargo.toml` file
- Once that is done, push the code to main, and a release workflow will get triggered which would build multi platform binaries and distribute them via multiple channels.g

## License

This project is licensed under the MIT License - see the LICENSE file for details.
