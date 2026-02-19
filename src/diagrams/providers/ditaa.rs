use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct DitaaProvider {
    pub bin_path: PathBuf,
}

impl DitaaProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for DitaaProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // ditaa -jar ditaa.jar input output
        // OR ditaa input output if it's a wrapper script
        // Config bin_path points to the executable.

        // Support only specific formats
        if format != "png" && format != "svg" {
            // ditaa mostly produces PNG. SVG might be supported by recent versions or ditaa-mini/similar?
            // Original ditaa is PNG (and EPS?).
            // Some forks support SVG.
            // Let's assume defaults for now. If format is not png, warn or fail?
            // Users might want SVG. If ditaa supports --svg or similar.
            // Standard ditaa does NOT support SVG.
            // Kroki uses `ditaa` (java) -> PNG. output must be png.
            // If user asks for SVG, we might fail.
            if format == "svg" {
                return Err(anyhow::anyhow!(
                    "Ditaa provider only supports PNG format (standard ditaa limitation)"
                ));
            }
        }

        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        let suffix = format!(".{}", format);
        let mut output_file_builder = tempfile::Builder::new();
        output_file_builder.suffix(&suffix);
        let output_file_with_ext = output_file_builder
            .tempfile()
            .context("Failed to create temp output file")?;
        let output_path = output_file_with_ext.path().to_path_buf();

        let mut cmd = Command::new(&self.bin_path);

        // ditaa input output
        cmd.arg(input_file.path());
        cmd.arg(&output_path);

        // ditaa options?

        let output = cmd.output().context("Failed to execute ditaa")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Ditaa conversion failed: {}", stderr));
        }

        let mut result = Vec::new();
        let mut file = std::fs::File::open(&output_path).context("Failed to open output file")?;
        std::io::Read::read_to_end(&mut file, &mut result).context("Failed to read output file")?;

        Ok(result)
    }
}
