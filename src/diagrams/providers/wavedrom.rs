use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct WavedromProvider {
    pub bin_path: PathBuf,
}

impl WavedromProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for WavedromProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // Wavedrom CLI: wavedrom-cli -i input.json -s output.svg
        // Does not support stdin/stdout well in all versions.

        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        // We need a path for output.
        // Wavedrom CLI infers format from extension?
        // Help says: -s, --svg path to generated SVG.

        let output_file = NamedTempFile::new().context("Failed to create temporary output file")?;
        let output_path = output_file.path().to_path_buf(); // Keep path, but file might be deleted/overwritten?

        // NamedTempFile deletes on drop. We can perform operation on its path.

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("-i").arg(input_file.path());

        match format {
            "svg" => {
                cmd.arg("-s").arg(&output_path);
            }
            "png" => {
                cmd.arg("-p").arg(&output_path);
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported format for Wavedrom: {}",
                    format
                ))
            }
        }

        let output = cmd.output().context("Failed to execute wavedrom-cli")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Wavedrom conversion failed: {}", stderr));
        }

        // Read output file
        let mut result = Vec::new();
        let mut file = std::fs::File::open(&output_path).context("Failed to open output file")?;
        file.read_to_end(&mut result)
            .context("Failed to read output file")?;

        Ok(result)
    }
}
