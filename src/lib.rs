// Public API
pub use cli::Args;
pub use k8s::K8sClient;

// Internal modules
mod cli;
mod k8s;
mod utils;

// Re-export commonly used items
pub use k8s::{extract_registry, process_pod, split_image, K8sError};

pub use utils::display_pod_images;
pub use utils::logging;

pub use cli::{Commands, GetImages, LogFormat, OutputFormat};
