use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A provider for Mermaid diagrams using the `mmdc` CLI tool.
pub struct MermaidProvider {
    /// The path to the `mmdc` binary.
    pub bin_path: PathBuf,
}

impl MermaidProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for MermaidProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(()) // Todo: implement validation logic
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // mmdc requires an input file or stdin, and output file.
        // It's less stream-friendly than dot.
        // We'll use a temporary file for output if needed, or see if it supports stdout.
        // mmdc -i - -o - (output to stdout not fully supported in all versions, let's check docs or try)
        // Modern mmdc supports `-o -` for stdout? Let's try.
        // If not, use tempfile.

        // Create a temporary input file because passing via stdin to mmdc can be flaky with some shells?
        // Actually, let's try stdin first with `-i -`

        let mut child = Command::new(&self.bin_path)
            .args(["-i", "-", "-o", "-"]) // Output to stdout
            .arg(match format {
                "svg" => "--outputFormat=svg",
                "png" => "--outputFormat=png",
                "pdf" => "--outputFormat=pdf",
                _ => "--outputFormat=svg",
            })
            // .arg("--puppeteerConfigFile=...") // TODO: Support config
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn mmdc")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Fallback for mmdc error
            Err(anyhow::anyhow!("Mermaid conversion failed: {}", stderr))
        }
    }
}
