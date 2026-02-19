use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct ExcalidrawProvider {
    pub bin_path: PathBuf,
}

impl ExcalidrawProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for ExcalidrawProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        // Validation could check if it's valid JSON
        Ok(())
    }

    fn generate(&self, source: &str, _format: &str) -> Result<Vec<u8>> {
        // excalidraw-to-svg usually expects a file input.
        // We'll write source to a temp file.
        let mut temp_input = NamedTempFile::new()?;
        temp_input.write_all(source.as_bytes())?;
        let input_path = temp_input.path().to_str().unwrap().to_string();

        // Output to stdout or another temp file?
        // Let's try capturing stdout first.
        // Usage might be: excalidraw-to-svg <input>

        let output = Command::new(&self.bin_path)
            .arg(&input_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute excalidraw-to-svg")?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(anyhow::anyhow!(
                    "Excalidraw conversion succeeded but output is empty"
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Excalidraw conversion failed: {}", stderr))
        }
    }
}
