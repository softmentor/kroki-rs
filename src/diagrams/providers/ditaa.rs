use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;

crate::diagrams::define_provider!(DitaaProvider);

#[async_trait]
impl DiagramProvider for DitaaProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        if format != "png" {
            return Err(DiagramError::UnsupportedFormat {
                format: format.into(),
                provider: "Ditaa".into(),
            });
        }

        let mut input_file = NamedTempFile::new().map_err(|e| {
            DiagramError::Internal(format!("Failed to create temporary input file: {}", e))
        })?;
        input_file.write_all(source.as_bytes()).map_err(|e| {
            DiagramError::Internal(format!("Failed to write source to temp file: {}", e))
        })?;

        let suffix = format!(".{}", format);
        let mut output_file_builder = tempfile::Builder::new();
        output_file_builder.suffix(&suffix);
        let output_file_with_ext = output_file_builder.tempfile().map_err(|e| {
            DiagramError::Internal(format!("Failed to create temp output file: {}", e))
        })?;
        let output_path = output_file_with_ext.path().to_path_buf();

        // Detect if the binary is actually a JAR file (common in Debian/Ubuntu)
        let is_jar = std::fs::File::open(&self.bin_path)
            .map(|mut f| {
                let mut buf = [0u8; 2];
                use std::io::Read;
                let _ = f.read_exact(&mut buf);
                buf == *b"PK"
            })
            .unwrap_or(false);

        let mut cmd = if is_jar {
            let mut c = Command::new("java");
            c.arg("-Djava.awt.headless=true")
                .arg("-jar")
                .arg(&self.bin_path);
            c
        } else {
            Command::new(&self.bin_path)
        };

        cmd.arg(input_file.path());
        cmd.arg(&output_path);
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

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
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(DiagramError::ProcessFailed(format!(
                "Ditaa conversion failed. stderr: {}, stdout: {}",
                stderr, stdout
            )));
        }

        let result = tokio::fs::read(&output_path).await.map_err(|e| {
            DiagramError::Internal(format!("Failed to read ditaa output file: {}", e))
        })?;

        Ok(result)
    }
}
