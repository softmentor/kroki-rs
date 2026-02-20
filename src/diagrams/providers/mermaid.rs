use crate::diagrams::DiagramProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(MermaidProvider);

#[async_trait]
impl DiagramProvider for MermaidProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(()) // Todo: implement validation logic
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // mmdc requires an input file or stdin, and output file.
        // It's less stream-friendly than dot.
        // We'll use a temporary file for output if needed, or see if it supports stdout.
        // mmdc -i - -o - (output to stdout not fully supported in all versions, let's check docs or try)
        // Modern mmdc supports `-o -` for stdout? Let's try.
        // If not, use tempfile.

        // Create a temporary input file because passing via stdin to mmdc can be flaky with some shells?
        // Actually, let's try stdin first with `-i -`

        let mut cmd = Command::new(&self.bin_path);
        cmd.args(["-i", "-", "-o", "-"]) // Output to stdout
            .arg(match format {
                "svg" => "--outputFormat=svg",
                "png" => "--outputFormat=png",
                "pdf" => "--outputFormat=pdf",
                _ => "--outputFormat=svg",
            })
            // .arg("--puppeteerConfigFile=...") // TODO: Support config
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
            if output.stdout.is_empty() {
                return Err(anyhow::anyhow!(
                    "Mermaid conversion succeeded but output is empty"
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Fallback for mmdc error
            Err(anyhow::anyhow!("Mermaid conversion failed: {}", stderr))
        }
    }
}
