use crate::cli::formats::OutputFormat;
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

pub fn display_pod_images(images: &[PodImage], output_format: &OutputFormat) {
    if images.is_empty() {
        warn!("No images found matching criteria");
        return;
    }

    let mut table = Table::new();
    // Set format to remove borders and extra spacing
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .separator(
            prettytable::format::LinePosition::Title,
            prettytable::format::LineSeparator::new('-', '-', '-', '-'),
        )
        .padding(0, 1)
        .build();
    table.set_format(format);

    let mut header_cells = Vec::new();

    header_cells.push("POD");
    header_cells.push("NAMESPACE");
    header_cells.push("CONTAINER");
    if matches!(output_format, OutputFormat::Wide) {
        header_cells.push("REGISTRY");
    }
    header_cells.push("IMAGE");
    header_cells.push("VERSION");
    if matches!(output_format, OutputFormat::Wide) {
        header_cells.push("DIGEST");
        header_cells.push("NODE");
    }

    let header_row = header_cells.into_iter().collect::<Vec<_>>();
    table.add_row(prettytable::Row::new(
        header_row.into_iter().map(prettytable::Cell::new).collect(),
    ));

    for image in images {
        let mut row = prettytable::Row::new(Vec::new());

        row.add_cell(prettytable::Cell::new(&image.pod_name));
        row.add_cell(prettytable::Cell::new(&image.namespace));
        row.add_cell(prettytable::Cell::new(&image.container_name));
        if matches!(output_format, OutputFormat::Wide) {
            row.add_cell(prettytable::Cell::new(&image.registry).style_spec("Fy"));
        }
        row.add_cell(prettytable::Cell::new(&image.image_name));
        row.add_cell(prettytable::Cell::new(&image.image_version));
        if matches!(output_format, OutputFormat::Wide) {
            row.add_cell(prettytable::Cell::new(&image.digest));
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
