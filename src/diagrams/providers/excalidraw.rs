use crate::diagrams::DiagramProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(ExcalidrawProvider);

#[async_trait]
impl DiagramProvider for ExcalidrawProvider {
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" {
            return Err(anyhow::anyhow!(
                "Excalidraw provider only supports SVG format, got '{}'",
                format
            ));
        }

        let mut temp_input = NamedTempFile::new()?;
        temp_input.write_all(source.as_bytes())?;
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
