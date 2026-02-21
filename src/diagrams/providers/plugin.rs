use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// A diagram provider that executes an external plugin via a subprocess protocol.
///
/// It communicates with the plugin using stdin (optional) and captures stdout
/// for the rendered diagram. It supports argument templating for formats.
pub struct PluginProvider {
    /// Human-readable name of the plugin (e.g. "mytool").
    pub name: String,
    /// Path to the executable binary.
    pub command: String,
    /// Arguments passed to the binary. Supports `{format}` substitution.
    pub args: Vec<String>,
    /// Whether to pipe the diagram source to the binary's stdin.
    pub stdin: bool,
    /// Maximum time allowed for the plugin to run.
    pub timeout_ms: u64,
}

impl PluginProvider {
    /// Creates a new plugin provider from the given configuration.
    pub fn new(config: &crate::config::PluginConfig) -> Self {
        Self {
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            stdin: config.stdin,
            timeout_ms: config.timeout_ms.unwrap_or(5000),
        }
    }
}

#[async_trait]
impl DiagramProvider for PluginProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        let mut cmd = Command::new(&self.command);

        // Template substitution for {format}
        for arg in &self.args {
            cmd.arg(arg.replace("{format}", format));
        }

        if self.stdin {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let input_bytes = if self.stdin {
            Some(source.as_bytes())
        } else {
            None
        };

        let output = crate::diagrams::run_process_with_timeout(
            &self.name,
            cmd,
            input_bytes,
            Some(self.timeout_ms),
            source.len(),
        )
        .await?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(DiagramError::ProcessFailed(format!(
                    "Plugin '{}' succeeded but returned empty output",
                    self.name
                )));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DiagramError::ProcessFailed(format!(
                "Plugin '{}' failed: {}",
                self.name, stderr
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_provider_stdout() {
        let config = crate::config::PluginConfig {
            name: "test-echo".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            stdin: false,
            formats: vec!["txt".to_string()],
            timeout_ms: Some(1000),
        };
        let provider = PluginProvider::new(&config);
        let result = provider.generate("", "txt").await.unwrap();
        assert_eq!(String::from_utf8_lossy(&result).trim(), "hello");
    }

    #[tokio::test]
    async fn test_plugin_provider_stdin() {
        // Use 'cat' as a simple plugin that echoes stdin to stdout
        let config = crate::config::PluginConfig {
            name: "test-cat".to_string(),
            command: "cat".to_string(),
            args: vec![],
            stdin: true,
            formats: vec!["txt".to_string()],
            timeout_ms: Some(1000),
        };
        let provider = PluginProvider::new(&config);
        let result = provider.generate("input-text", "txt").await.unwrap();
        assert_eq!(String::from_utf8_lossy(&result), "input-text");
    }
}
