# Kelper

A Rust-based CLI tool to query Kubernetes pod images and their registries.

## Features

- List all pods in a specified namespace
- Extract container images from pods
- Identify container registries
- Colored output for better readability

## Prerequisites

- Rust 1.70 or later
- Access to a Kubernetes cluster
- `kubectl` configured with your cluster

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/kelper.git
cd kelper

# Build the project
cargo build --release
```

## Usage

```bash
# Run with default namespace (default)
./target/release/kelper

# Run with a specific namespace
./target/release/kelper -n kube-system

# Run with the new command structure
kelper get images -n <namespace>
```

## Output Format

The tool will display:

- Pod name
- Container image
- Container registry

## License

MIT
