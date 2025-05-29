use crate::k8s::PodImage;
use colored::*;
use prettytable::Table;

pub fn display_pod_images(
    images: &[PodImage],
    show_node: bool,
    show_namespace: bool,
    show_pod: bool,
) {
    if images.is_empty() {
        println!("{}", "No images found matching criteria.".yellow());
        return;
    }
    println!("\n{}", "Pod Images and Registries:".green().bold());
    println!("{}", "=".repeat(80));

    let mut table = Table::new();
    let mut header_cells = Vec::new();

    if show_pod {
        header_cells.push("POD NAME");
    }

    if show_node {
        header_cells.push("NODE");
    }

    if show_namespace {
        header_cells.push("NAMESPACE");
    }

    header_cells.extend(vec!["CONTAINER", "IMAGE NAME", "VERSION", "REGISTRY"]);

    let header_row = header_cells.into_iter().collect::<Vec<_>>();
    table.add_row(prettytable::Row::new(
        header_row.into_iter().map(prettytable::Cell::new).collect(),
    ));

    for image in images {
        let mut row = prettytable::Row::new(Vec::new());

        if show_pod {
            row.add_cell(prettytable::Cell::new(&image.pod_name));
        }
        if show_node {
            row.add_cell(prettytable::Cell::new(&image.node_name));
        }
        if show_namespace {
            row.add_cell(prettytable::Cell::new(&image.namespace));
        }

        row.add_cell(prettytable::Cell::new(&image.container_name));
        row.add_cell(prettytable::Cell::new(&image.image_name));
        row.add_cell(prettytable::Cell::new(&image.image_version));
        row.add_cell(prettytable::Cell::new(&image.registry).style_spec("Fy"));

        table.add_row(row);
    }

    table.printstd();
    println!("\n{}", "=".repeat(80));
}
