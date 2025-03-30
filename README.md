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

```bash
cargo install kelper
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

#### Image Processing Tests

- Registry extraction from various image formats
- Image name and version splitting
- Handling of different registry types (Docker Hub, GCR, Quay.io, private registries)

#### Pod Processing Tests

- Basic pod with multiple containers
- Pods without spec
- Pods with empty containers
- Containers without images
- Complex image paths
- Private registry images

#### Test Coverage

The tests cover various scenarios including:

- Edge cases (missing fields, empty values)
- Different image formats and registries
- Error conditions
- Data structure validation

To run the tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_process_pod
```

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality. To set them up:

1. Install pre-commit:

```bash
pip install pre-commit
```

2. Install the git hooks:

```bash
pre-commit install
```

The pre-commit hooks will run automatically on every commit, checking for:

- Code formatting (rustfmt)
- Linting (clippy)
- Security vulnerabilities (cargo-audit)
- And more...

## License

This project is licensed under the MIT License - see the LICENSE file for details.
