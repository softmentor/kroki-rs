use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(CommandProvider);

#[async_trait]
impl DiagramProvider for CommandProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, _format: &str) -> DiagramResult<Vec<u8>> {
        let mut cmd = Command::new(&self.bin_path);
        cmd.arg(format!("-T{}", _format))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = crate::diagrams::run_process_with_timeout(
            "dot",
            cmd,
            Some(source.as_bytes()),
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(DiagramError::ProcessFailed(
                    "Command succeeded but returned empty output".into(),
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DiagramError::ProcessFailed(format!(
                "Command failed: {}",
                stderr
            )))
        }
    }
}
