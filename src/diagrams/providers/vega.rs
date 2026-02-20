use crate::diagrams::DiagramProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(VegaProvider);

#[async_trait]
impl DiagramProvider for VegaProvider {
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" {
            return Err(anyhow::anyhow!(
                "Vega support currently limited to SVG format, got '{}'",
                format
            ));
        }

        let mut cmd = Command::new(&self.bin_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = crate::diagrams::run_process_with_timeout(
            "vg2svg",
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
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        if format != "svg" {
            return Err(anyhow::anyhow!(
                "Vega-Lite support currently limited to SVG"
            ));
        }

        // Stage 1: vl2vg (Vega-Lite spec → Vega spec)
        let mut vl_cmd = Command::new(&self.vl_bin_path);
        vl_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let vl_output = crate::diagrams::run_process_with_timeout(
            "vl2vg",
            vl_cmd,
            Some(source.as_bytes()),
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if !vl_output.status.success() {
            let stderr = String::from_utf8_lossy(&vl_output.stderr);
            return Err(anyhow::anyhow!("vl2vg (stage 1) failed: {}", stderr));
        }

        // Stage 2: vg2svg (Vega spec → SVG)
        let vg_input = &vl_output.stdout;
        let mut vg_cmd = Command::new(&self.vg_bin_path);
        vg_cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let vg_output = crate::diagrams::run_process_with_timeout(
            "vg2svg",
            vg_cmd,
            Some(vg_input.as_slice()),
            self.timeout_ms,
            vg_input.len(),
        )
        .await?;

        if vg_output.status.success() {
            if vg_output.stdout.is_empty() {
                return Err(anyhow::anyhow!(
                    "vg2svg (stage 2) succeeded but output is empty (vl2vg produced {} bytes)",
                    vg_input.len()
                ));
            }
            Ok(vg_output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&vg_output.stderr);
            Err(anyhow::anyhow!(
                "vg2svg (stage 2) failed (vl2vg produced {} bytes): {}",
                vg_input.len(),
                stderr
            ))
        }
    }
}
