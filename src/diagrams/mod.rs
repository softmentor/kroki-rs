pub mod providers {
    pub mod bpmn;
    pub mod cmd;
    pub mod d2;
    pub mod ditaa;
    pub mod excalidraw;
    pub mod mermaid;
    pub mod plantuml;
    pub mod vega;
    pub mod wavedrom;
}
pub mod registry;

use anyhow::Result;
use async_trait::async_trait;

/// Computes an adaptive timeout based on input size.
/// Base: 3000ms. Adds 1000ms per 10KB of payload. Max default: 10000ms.
pub fn adaptive_timeout(source_len: usize) -> u64 {
    let base = 3000;
    let scaling = (source_len as u64 / 10240) * 1000;
    std::cmp::min(base + scaling, 10000)
}

/// A macro to drastically reduce boilerplate structurally identical providers.
#[macro_export]
macro_rules! define_provider {
    ($name:ident) => {
        pub struct $name {
            pub bin_path: std::path::PathBuf,
            pub timeout_ms: Option<u64>,
        }

        impl $name {
            pub fn new(bin_path: std::path::PathBuf, timeout_ms: Option<u64>) -> Self {
                Self {
                    bin_path,
                    timeout_ms,
                }
            }
        }
    };
}
pub(crate) use define_provider;

/// Safely executes a child process, automatically managing timeouts, input piping, and memory cleanup.
pub async fn run_process_with_timeout(
    mut cmd: tokio::process::Command,
    source: Option<&[u8]>,
    timeout_ms: Option<u64>,
    source_len: usize,
) -> Result<std::process::Output> {
    use anyhow::Context;
    use tokio::io::AsyncWriteExt;

    cmd.kill_on_drop(true);
    let mut child = cmd.spawn().context("Failed to spawn process")?;

    if let (Some(mut stdin), Some(src)) = (child.stdin.take(), source) {
        stdin
            .write_all(src)
            .await
            .context("Failed to write to stdin")?;
    }

    let actual_timeout = std::cmp::min(
        timeout_ms.unwrap_or_else(|| adaptive_timeout(source_len)),
        20000,
    );
    let output_future = child.wait_with_output();

    match tokio::time::timeout(
        std::time::Duration::from_millis(actual_timeout),
        output_future,
    )
    .await
    {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => anyhow::bail!("IO Error during execution: {}", e),
        Err(_) => anyhow::bail!("Process timed out after {}ms", actual_timeout),
    }
}

/// A trait for diagram generation providers.
///
/// Each provider implementation is responsible for a specific diagram type
/// (e.g., Mermaid, PlantUML, Graphviz).
#[async_trait]
pub trait DiagramProvider {
    /// Validates the diagram source text.
    ///
    /// Returns `Ok(())` if the source is valid, or an error otherwise.
    fn validate(&self, source: &str) -> Result<()>;

    /// Generates a diagram image from the source text.
    ///
    /// # Arguments
    /// * `source` - The diagram description text.
    /// * `format` - The desired output format (e.g., "svg", "png").
    ///
    /// Returns a `Vec<u8>` containing the image data.
    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::process::Command;

    #[tokio::test]
    async fn test_run_process_with_timeout() {
        let mut cmd = Command::new("sleep");
        cmd.arg("2"); // Sleep for 2 seconds

        // Set timeout to 100ms, which is much shorter than 2 seconds
        let result = run_process_with_timeout(cmd, None, Some(100), 0).await;

        // Ensure the function returns an error and it's specifically a timeout
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert_eq!(err_msg, "Process timed out after 100ms");
    }
}
