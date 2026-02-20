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
    fn validate(&self, _source: &str) -> Result<()> {
        // Validation could check if it's valid JSON
        Ok(())
    }

    async fn generate(&self, source: &str, _format: &str) -> Result<Vec<u8>> {
        // excalidraw-to-svg usually expects a file input.
        // We'll write source to a temp file.
        let mut temp_input = NamedTempFile::new()?;
        temp_input.write_all(source.as_bytes())?;
        let input_path = temp_input.path().to_str().unwrap().to_string();

        // Output to stdout or another temp file?
        // Let's try capturing stdout first.
        // Usage might be: excalidraw-to-svg <input>

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg(&input_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output =
            crate::diagrams::run_process_with_timeout(cmd, None, self.timeout_ms, source.len())
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
