use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A generic provider that wraps an external command-line tool.
///
/// This provider is used for tools that support stdin/stdout for diagram generation (e.g., Graphviz).
pub struct CommandProvider {
    /// The path to the executable binary.
    pub bin_path: PathBuf,
}

impl CommandProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for CommandProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(()) // Todo: implement validation logic
    }

    fn generate(&self, source: &str, _format: &str) -> Result<Vec<u8>> {
        let mut child = Command::new(&self.bin_path)
            .args(["-Tsvg"]) // Default to SVG for now, make configurable
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn command")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(anyhow::anyhow!(
                    "Command succeeded but returned empty output"
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Command failed: {}", stderr))
        }
    }
}
