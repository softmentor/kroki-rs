use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(WavedromProvider);

#[async_trait]
impl DiagramProvider for WavedromProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        let mut input_file = NamedTempFile::new().map_err(|e| {
            DiagramError::Internal(format!("Failed to create temporary input file: {}", e))
        })?;
        input_file.write_all(source.as_bytes()).map_err(|e| {
            DiagramError::Internal(format!("Failed to write source to temp file: {}", e))
        })?;

        let output_file = NamedTempFile::new().map_err(|e| {
            DiagramError::Internal(format!("Failed to create temporary output file: {}", e))
        })?;
        let output_path = output_file.path().to_path_buf();

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("-i").arg(input_file.path());
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        match format {
            "svg" => {
                cmd.arg("-s").arg(&output_path);
            }
            "png" => {
                cmd.arg("-p").arg(&output_path);
            }
            _ => {
                return Err(DiagramError::UnsupportedFormat {
                    format: format.into(),
                    provider: "Wavedrom".into(),
                })
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
            return Err(DiagramError::ProcessFailed(format!(
                "Wavedrom conversion failed: {}",
                stderr
            )));
        }

        let result = tokio::fs::read(&output_path).await.map_err(|e| {
            DiagramError::Internal(format!("Failed to read Wavedrom output file: {}", e))
        })?;

        Ok(result)
    }
}
