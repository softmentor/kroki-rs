use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(WavedromProvider);

#[async_trait]
impl DiagramProvider for WavedromProvider {
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        let output_file = NamedTempFile::new().context("Failed to create temporary output file")?;
        let output_path = output_file.path().to_path_buf();

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
                    "Unsupported format for Wavedrom: '{}'. Supported: svg, png",
                    format
                ))
            }
        }

        let output = crate::diagrams::run_process_with_timeout(
            "wavedrom-cli",
            cmd,
            None,
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Wavedrom conversion failed: {}", stderr));
        }

        let result = tokio::fs::read(&output_path)
            .await
            .context("Failed to read Wavedrom output file")?;

        Ok(result)
    }
}
