use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(DitaaProvider);

#[async_trait]
impl DiagramProvider for DitaaProvider {
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "png" {
            return Err(anyhow::anyhow!(
                "Ditaa provider only supports PNG format (standard ditaa limitation), got '{}'",
                format
            ));
        }

        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        let suffix = format!(".{}", format);
        let mut output_file_builder = tempfile::Builder::new();
        output_file_builder.suffix(&suffix);
        let output_file_with_ext = output_file_builder
            .tempfile()
            .context("Failed to create temp output file")?;
        let output_path = output_file_with_ext.path().to_path_buf();

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg(input_file.path());
        cmd.arg(&output_path);

        let output = crate::diagrams::run_process_with_timeout(
            "ditaa",
            cmd,
            None,
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Ditaa conversion failed: {}", stderr));
        }

        let result = tokio::fs::read(&output_path)
            .await
            .context("Failed to read ditaa output file")?;

        Ok(result)
    }
}
