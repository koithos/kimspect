use crate::k8s::PodImage;
use prettytable::Table;
use tracing::warn;

pub mod logging;

pub const KNOWN_REGISTRIES: [&str; 11] = [
    "docker.io",
    "registry.hub.docker.com",
    "ghcr.io",
    "gcr.io",
    "quay.io",
    "registry.gitlab.com",
    "mcr.microsoft.com",
    "registry.k8s.io",
    "public.ecr.aws",
    "docker.pkg.github.com",
    "pkg.dev",
];

pub fn display_pod_images(
    images: &[PodImage],
    show_node: bool,
    show_namespace: bool,
    show_pod: bool,
) {
    if images.is_empty() {
        warn!("No images found matching criteria");
        return;
    }

    let mut table = Table::new();
    // Set format to remove borders
    table.set_format(*prettytable::format::consts::FORMAT_CLEAN);

    let mut header_cells = Vec::new();

    if show_pod {
        header_cells.push("POD");
    }
    if show_namespace {
        header_cells.push("NAMESPACE");
    }

    header_cells.extend(vec!["CONTAINER", "REGISTRY", "IMAGE", "VERSION", "DIGEST"]);

    if show_node {
        header_cells.push("NODE");
    }

    let header_row = header_cells.into_iter().collect::<Vec<_>>();
    table.add_row(prettytable::Row::new(
        header_row.into_iter().map(prettytable::Cell::new).collect(),
    ));

    for image in images {
        let mut row = prettytable::Row::new(Vec::new());

        if show_pod {
            row.add_cell(prettytable::Cell::new(&image.pod_name));
        }
        if show_namespace {
            row.add_cell(prettytable::Cell::new(&image.namespace));
        }

        row.add_cell(prettytable::Cell::new(&image.container_name));
        row.add_cell(prettytable::Cell::new(&image.registry).style_spec("Fy"));
        row.add_cell(prettytable::Cell::new(&image.image_name));
        row.add_cell(prettytable::Cell::new(&image.image_version));
        row.add_cell(prettytable::Cell::new(&image.digest));

        if show_node {
            row.add_cell(prettytable::Cell::new(&image.node_name));
        }

        table.add_row(row);
    }

    table.printstd();
}

/// Strips the registry prefix from an image name if it exists.
///
/// # Arguments
///
/// * `image_name` - The full image name that may include a registry prefix
/// * `registry` - The registry to strip from the image name
///
/// # Returns
///
/// The image name without the registry prefix
pub fn strip_registry(image_name: &str, registry: &str) -> String {
    if image_name.starts_with(registry) {
        image_name
            .strip_prefix(&format!("{}/", registry))
            .unwrap_or(image_name)
            .to_string()
    } else {
        image_name.to_string()
    }
}
