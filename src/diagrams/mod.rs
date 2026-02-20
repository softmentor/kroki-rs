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
///
/// # Arguments
/// * `tool_name` - Human-readable name of the tool (e.g., "mmdc", "dot") for error messages.
/// * `cmd` - The tokio Command to execute.
/// * `source` - Optional bytes to pipe to stdin.
/// * `timeout_ms` - Optional explicit timeout; falls back to adaptive_timeout.
/// * `source_len` - Length of the source input (used for adaptive timeout and error context).
pub async fn run_process_with_timeout(
    tool_name: &str,
    mut cmd: tokio::process::Command,
    source: Option<&[u8]>,
    timeout_ms: Option<u64>,
    source_len: usize,
) -> Result<std::process::Output> {
    use anyhow::Context;
    use tokio::io::AsyncWriteExt;

    cmd.kill_on_drop(true);
    let mut child = cmd.spawn().context(format!(
        "Failed to spawn '{}'. Is the tool installed and in your PATH?",
        tool_name
    ))?;

    if let (Some(mut stdin), Some(src)) = (child.stdin.take(), source) {
        stdin
            .write_all(src)
            .await
            .context(format!("Failed to write to stdin of '{}'", tool_name))?;
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
        Ok(Err(e)) => anyhow::bail!(
            "'{}' IO error (input: {} bytes): {}",
            tool_name,
            source_len,
            e
        ),
        Err(_) => anyhow::bail!(
            "'{}' timed out after {}ms (input: {} bytes). Consider increasing the timeout in kroki.toml.",
            tool_name,
            actual_timeout,
            source_len
        ),
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
        let result = run_process_with_timeout("sleep", cmd, None, Some(100), 0).await;

        // Ensure the function returns an error and it's specifically a timeout
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert_eq!(err_msg, "'sleep' timed out after 100ms (input: 0 bytes). Consider increasing the timeout in kroki.toml.");
    }
}
