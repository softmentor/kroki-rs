use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(ExcalidrawProvider);

#[async_trait]
impl DiagramProvider for ExcalidrawProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        if format != "svg" {
            return Err(DiagramError::UnsupportedFormat {
                format: format.into(),
                provider: "Excalidraw".into(),
            });
        }

        let mut temp_input = NamedTempFile::new().map_err(|e| {
            DiagramError::Internal(format!("Failed to create temporary input file: {}", e))
        })?;
        temp_input.write_all(source.as_bytes()).map_err(|e| {
            DiagramError::Internal(format!("Failed to write source to temp file: {}", e))
        })?;
        let input_path = temp_input.path().to_str().unwrap().to_string();

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg(&input_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = crate::diagrams::run_process_with_timeout(
            "excalidraw-to-svg",
            cmd,
            None,
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(DiagramError::ProcessFailed(
                    "Excalidraw conversion succeeded but output is empty".into(),
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DiagramError::ProcessFailed(format!(
                "Excalidraw conversion failed: {}",
                stderr
            )))
        }
    }
}
