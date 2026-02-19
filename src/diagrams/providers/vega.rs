use crate::diagrams::DiagramProvider;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct VegaProvider {
    pub bin_path: PathBuf, // vg2svg
}

impl VegaProvider {
    pub fn new(bin_path: PathBuf) -> Self {
        Self { bin_path }
    }
}

impl DiagramProvider for VegaProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" && format != "png" && format != "pdf" {
            return Err(anyhow::anyhow!("Unsupported format for Vega: {}", format));
        }

        // vg2png, vg2pdf, vg2svg sharing the same binary base usually?
        // Actually vega-cli provides vg2svg, vg2png, vg2pdf.
        // But for now let's assume valid config points to vg2svg.
        // If users want PNG, they might need vg2png.
        // For simplicity, let's assume we stick to vg2svg for now or handle format in bin_path logic?
        // Wait, the plan was simple CLI wrappers.
        // Let's just use vg2svg for SVG. If format is PNG, we might need a different tool or flag.
        // vega-cli: vg2png exists.

        // If configured as vg2svg but requested png, we might be in trouble if we don't swap the binary.
        // For this iteration, let's assume we only strictly support SVG via vg2svg,
        // OR we try to deduce the sibling binary.

        // Let's implement SVG support first.
        if format != "svg" {
            return Err(anyhow::anyhow!("Vega support currently limited to SVG"));
        }

        let mut child = Command::new(&self.bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn vg2svg")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Vega conversion failed: {}", stderr))
        }
    }
}

pub struct VegaLiteProvider {
    pub vl_bin_path: PathBuf, // vl2vg
    pub vg_bin_path: PathBuf, // vg2svg
}

impl VegaLiteProvider {
    pub fn new(vl_bin_path: PathBuf, vg_bin_path: PathBuf) -> Self {
        Self {
            vl_bin_path,
            vg_bin_path,
        }
    }
}

impl DiagramProvider for VegaLiteProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" {
            return Err(anyhow::anyhow!(
                "Vega-Lite support currently limited to SVG"
            ));
        }

        // 1. Run vl2vg
        let mut vl_child = Command::new(&self.vl_bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn vl2vg")?;

        if let Some(mut stdin) = vl_child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }

        let vl_output = vl_child.wait_with_output()?;
        if !vl_output.status.success() {
            let stderr = String::from_utf8_lossy(&vl_output.stderr);
            return Err(anyhow::anyhow!("vl2vg failed: {}", stderr));
        }

        // 2. Run vg2svg with output of step 1
        let mut vg_child = Command::new(&self.vg_bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn vg2svg")?;

        if let Some(mut stdin) = vg_child.stdin.take() {
            stdin.write_all(&vl_output.stdout)?;
        }

        let vg_output = vg_child.wait_with_output()?;
        if vg_output.status.success() {
            Ok(vg_output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&vg_output.stderr);
            Err(anyhow::anyhow!("vg2svg failed: {}", stderr))
        }
    }
}
