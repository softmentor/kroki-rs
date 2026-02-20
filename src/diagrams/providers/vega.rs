use crate::diagrams::DiagramProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(VegaProvider);

#[async_trait]
impl DiagramProvider for VegaProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
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

        let mut cmd = Command::new(&self.bin_path);
        cmd.stdin(Stdio::piped())
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
                    "Vega conversion succeeded but output is empty"
                ));
            }
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
    pub timeout_ms: Option<u64>,
}

impl VegaLiteProvider {
    pub fn new(vl_bin_path: PathBuf, vg_bin_path: PathBuf, timeout_ms: Option<u64>) -> Self {
        Self {
            vl_bin_path,
            vg_bin_path,
            timeout_ms,
        }
    }
}

#[async_trait]
impl DiagramProvider for VegaLiteProvider {
    fn validate(&self, _source: &str) -> Result<()> {
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" {
            return Err(anyhow::anyhow!(
                "Vega-Lite support currently limited to SVG"
            ));
        }

        // 1. Run vl2vg
        let mut vl_cmd = Command::new(&self.vl_bin_path);
        vl_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let vl_output = crate::diagrams::run_process_with_timeout(
            vl_cmd,
            Some(source.as_bytes()),
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if !vl_output.status.success() {
            let stderr = String::from_utf8_lossy(&vl_output.stderr);
            return Err(anyhow::anyhow!("vl2vg failed: {}", stderr));
        }

        // 2. Run vg2svg with output of step 1
        let mut vg_cmd = Command::new(&self.vg_bin_path);
        vg_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let vg_output = crate::diagrams::run_process_with_timeout(
            vg_cmd,
            Some(&vl_output.stdout),
            self.timeout_ms,
            source.len(),
        )
        .await?;
        if vg_output.status.success() {
            if vg_output.stdout.is_empty() {
                return Err(anyhow::anyhow!("vg2svg succeeded but output is empty"));
            }
            Ok(vg_output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&vg_output.stderr);
            Err(anyhow::anyhow!("vg2svg failed: {}", stderr))
        }
    }
}
