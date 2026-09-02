use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_output_path(input: &Path) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));

    let stem = input
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("document");

    parent.join(format!("{stem}_compressed.pdf"))
}

pub fn file_size(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|error| {
            format!(
                "Impossible de lire les informations du fichier '{}': {error}",
                path.display()
            )
        })
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;

    let bytes = bytes as f64;

    if bytes >= MB {
        format!("{:.2} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes / KB)
    } else {
        format!("{bytes:.0} B")
    }
}
