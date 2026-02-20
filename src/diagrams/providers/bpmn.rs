use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(BpmnProvider);

#[async_trait]
impl DiagramProvider for BpmnProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        let mut cmd = Command::new(&self.bin_path);

        let suffix = format!(".{}", format);
        let mut output_file_builder = tempfile::Builder::new();
        output_file_builder.suffix(&suffix);
        let output_file_with_ext = output_file_builder
            .tempfile()
            .context("Failed to create temp output file with extension")?;
        let output_path_with_ext = output_file_with_ext.path().to_path_buf();

        let io_arg_ext = format!(
            "{}:{}",
            input_file.path().to_string_lossy(),
            output_path_with_ext.to_string_lossy()
        );

        cmd.arg(io_arg_ext);

        let output = crate::diagrams::run_process_with_timeout(
            "bpmn-to-image",
            cmd,
            None,
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("BPMN conversion failed: {}", stderr));
        }

        let result = tokio::fs::read(&output_path_with_ext)
            .await
            .context("Failed to read BPMN output file")?;

        if result.is_empty() {
            return Err(anyhow::anyhow!(
                "BPMN conversion succeeded but output file is empty"
            ));
        }

        Ok(result)
    }
}
