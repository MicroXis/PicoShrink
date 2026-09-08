use std::fs;
use std::path::{Path, PathBuf};

pub fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_output_path_adds_compressed_suffix() {
        let input = Path::new("document.pdf");

        let output = generate_output_path(input);

        assert_eq!(output, PathBuf::from("document_compressed.pdf"));
    }

    #[test]
    fn generate_output_path_preserves_parent_directory() {
        let input = Path::new("/tmp/documents/report.pdf");

        let output = generate_output_path(input);

        assert_eq!(
            output,
            PathBuf::from("/tmp/documents/report_compressed.pdf")
        );
    }

    #[test]
    fn generate_output_path_handles_multiple_dots() {
        let input = Path::new("rapport.final.v2.pdf");

        let output = generate_output_path(input);

        assert_eq!(output, PathBuf::from("rapport.final.v2_compressed.pdf"));
    }

    #[test]
    fn accepts_pdf_extension() {
        assert!(is_pdf(Path::new("document.pdf")));
    }

    #[test]
    fn accepts_mixed_case_pdf_extension() {
        assert!(is_pdf(Path::new("document.PdF")));
    }

    #[test]
    fn rejects_pdf_when_it_is_not_the_final_extension() {
        assert!(!is_pdf(Path::new("document.pdf.txt")));
    }

    #[test]
    fn accepts_uppercase_pdf_extension() {
        assert!(is_pdf(Path::new("document.PDF")));
    }

    #[test]
    fn rejects_non_pdf_extension() {
        assert!(!is_pdf(Path::new("document.txt")));
    }

    #[test]
    fn rejects_missing_extension() {
        assert!(!is_pdf(Path::new("document")));
    }

    #[test]
    fn format_file_size_formats_bytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1), "1 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn format_file_size_formats_kilobytes() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(10 * 1024), "10.00 KB");
    }

    #[test]
    fn format_file_size_formats_megabytes() {
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");

        assert_eq!(format_file_size(5 * 1024 * 1024), "5.00 MB");
    }

    #[test]
    fn format_file_size_switches_from_kilobytes_to_megabytes() {
        assert_eq!(format_file_size(1024 * 1024 - 1), "1024.00 KB");

        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn file_size_returns_correct_size() {
        let path = std::env::temp_dir().join("picoshrink_test_file_size.txt");

        fs::write(&path, b"hello").expect("failed to create test file");

        let size = file_size(&path).expect("file_size should succeed");

        assert_eq!(size, 5);

        fs::remove_file(&path).expect("failed to remove test file");
    }

    #[test]
    fn file_size_returns_error_for_missing_file() {
        let path = std::env::temp_dir().join("picoshrink_missing_file.pdf");

        let _ = fs::remove_file(&path);

        let result = file_size(&path);

        assert!(result.is_err());
    }
}
