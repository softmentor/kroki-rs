use crate::diagrams::DiagramProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(PlantUmlProvider);

#[async_trait]
impl DiagramProvider for PlantUmlProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // PlantUML CLI: java -jar plantuml.jar -pipe -tsvg
        // Or if installed via brew: plantuml -pipe -tsvg

        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("-pipe")
            .arg(match format {
                "svg" => "-tsvg",
                "png" => "-tpng",
                "txt" => "-ttxt",
                _ => "-tsvg",
            })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = crate::diagrams::run_process_with_timeout(
            cmd,
            Some(source.as_bytes()),
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("PlantUML conversion failed: {}", stderr))
        }
    }
}
