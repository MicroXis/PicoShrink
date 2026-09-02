use crate::models::CompressionLevel;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Ghostscript {
    executable: PathBuf,
    version: String,
}

impl Ghostscript {
    pub fn detect() -> Option<Self> {
        for candidate in Self::candidates() {
            if let Ok(version) = Self::get_version(&candidate) {
                return Some(Self {
                    executable: candidate,
                    version,
                });
            }
        }

        None
    }

    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        let version = Self::get_version(&path)?;

        Ok(Self {
            executable: path,
            version,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    fn find_in_path(name: &str) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;

        for directory in env::split_paths(&path) {
            let candidate = directory.join(name);

            if candidate.is_file() {
                return Some(candidate);
            }
        }

        None
    }

    fn candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Some(path) = Self::find_in_path("gs") {
                candidates.push(path);
            }

            candidates.push(PathBuf::from("/usr/bin/gs"));
            candidates.push(PathBuf::from("/usr/local/bin/gs"));
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(path) = Self::find_in_path("gs") {
                candidates.push(path);
            }

            candidates.push(PathBuf::from("/opt/homebrew/bin/gs"));
            candidates.push(PathBuf::from("/usr/local/bin/gs"));
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(path) = Self::find_in_path("gswin64c.exe") {
                candidates.push(path);
            }

            if let Some(path) = Self::find_in_path("gswin32c.exe") {
                candidates.push(path);
            }
        }

        candidates
    }

    fn get_version(path: &Path) -> Result<String, String> {
        let output = Command::new(path)
            .arg("--version")
            .output()
            .map_err(|error| format!("Impossible d'exécuter '{}': {error}", path.display()))?;

        if !output.status.success() {
            return Err(format!(
                "'{}' ne semble pas être un exécutable Ghostscript valide",
                path.display()
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if version.is_empty() {
            return Err(format!(
                "Impossible de récupérer la version depuis '{}'",
                path.display()
            ));
        }

        Ok(version)
    }

    pub fn compress(
        &self,
        input: &Path,
        output: &Path,
        level: CompressionLevel,
    ) -> Result<(), String> {
        if !input.is_file() {
            return Err(format!(
                "Le fichier d'entrée '{}' n'existe pas",
                input.display()
            ));
        }

        if input == output {
            return Err(
                "Le fichier de sortie ne peut pas être identique au fichier d'entrée".into(),
            );
        }

        let preset = match level {
            CompressionLevel::HighQuality => "/prepress",
            CompressionLevel::Balanced => "/ebook",
            CompressionLevel::Strong => "/screen",
        };

        let status = Command::new(&self.executable)
            .arg("-sDEVICE=pdfwrite")
            .arg("-dCompatibilityLevel=1.4")
            .arg(format!("-dPDFSETTINGS={preset}"))
            .arg("-dNOPAUSE")
            .arg("-dQUIET")
            .arg("-dBATCH")
            .arg("-dDetectDuplicateImages=true")
            .arg("-dCompressFonts=true")
            .arg("-dSubsetFonts=true")
            .arg(format!("-sOutputFile={}", output.display()))
            .arg(input)
            .status()
            .map_err(|error| {
                format!(
                    "Impossible de lancer Ghostscript '{}': {error}",
                    self.executable.display()
                )
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Ghostscript a échoué avec le code {:?}",
                status.code()
            ))
        }
    }
}
