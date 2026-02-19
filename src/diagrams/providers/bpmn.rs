use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct BpmnProvider {
    pub bin_path: PathBuf,
}

impl BpmnProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for BpmnProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // bpmn-to-image input.bpmn:output.png

        let mut input_file =
            NamedTempFile::new().context("Failed to create temporary input file")?;
        input_file
            .write_all(source.as_bytes())
            .context("Failed to write source to temp file")?;

        // Construct argument: input_path:output_path
        // Remove unused io_arg block

        let mut cmd = Command::new(&self.bin_path);

        // Fix temporary value dropped while borrowed
        let suffix = format!(".{}", format);
        let mut output_file_builder = tempfile::Builder::new();
        output_file_builder.suffix(&suffix);
        let output_file_with_ext = output_file_builder
            .tempfile()
            .context("Failed to create temp output file with extension")?;
        let output_path_with_ext = output_file_with_ext.path().to_path_buf();

        let io_arg_ext = format!(
            "{}:{}",
            input_file.path().to_string_lossy(),
            output_path_with_ext.to_string_lossy()
        );

        cmd.arg(io_arg_ext);

        // Add min-dimensions or other defaults?

        let output = cmd.output().context("Failed to execute bpmn-to-image")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("BPMN conversion failed: {}", stderr));
        }

        // Read output file
        let mut result = Vec::new();
        let mut file =
            std::fs::File::open(&output_path_with_ext).context("Failed to open output file")?;
        file.read_to_end(&mut result)
            .context("Failed to read output file")?;

        Ok(result)
    }
}
