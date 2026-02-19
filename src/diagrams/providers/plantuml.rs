use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct PlantUmlProvider {
    pub bin_path: PathBuf,
}

impl PlantUmlProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for PlantUmlProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // PlantUML CLI: java -jar plantuml.jar -pipe -tsvg
        // Or if installed via brew: plantuml -pipe -tsvg

        let mut child = Command::new(&self.bin_path)
            .arg("-pipe")
            .arg(match format {
                "svg" => "-tsvg",
                "png" => "-tpng",
                "txt" => "-ttxt",
                _ => "-tsvg",
            })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn plantuml")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("PlantUML conversion failed: {}", stderr))
        }
    }
}
