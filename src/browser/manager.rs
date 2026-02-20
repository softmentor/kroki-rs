use crate::diagrams::{DiagramError, DiagramResult};
use anyhow::Result;
use reqwest::Client;
use serde::Serialize;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// Manages the lifecycle of the Node.js headless browser daemon.
pub struct BrowserManager {
    port: u16,
    client: Client,
    worker_process: Mutex<Option<Child>>,
}

impl Drop for BrowserManager {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.worker_process.lock() {
            if let Some(mut child) = lock.take() {
                // Force kill the child process to avoid zombie browser daemons
                let _ = child.start_kill();
            }
        }
    }
}

impl BrowserManager {
    /// Launches the Node.js worker and binds an HTTP client to its dynamic port.
    ///
    /// # Arguments
    /// * `pool_size` - The maximum number of concurrent Playwright `BrowserContext` pages.
    /// * `context_ttl` - The maximum payload evaluations a single context can process
    ///   before it is systematically destroyed and replaced to mitigate memory leaks.
    pub async fn start(pool_size: usize, context_ttl: usize) -> Result<Self> {
        let mut cmd = Command::new("node");

        let worker_path = std::env::current_dir()?.join("src/browser/worker.js");
        if !worker_path.exists() {
            anyhow::bail!("Playwright worker script not found at {:?}", worker_path);
        }

        cmd.arg(worker_path)
            .env("KROKI_BROWSER_POOL_SIZE", pool_size.to_string())
            .env("KROKI_BROWSER_CONTEXT_TTL", context_ttl.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;

        // Extract the port assigned natively by Express
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        let startup_timeout = tokio::time::Duration::from_secs(15);

        let find_port = async {
            while let Some(line) = reader.next_line().await? {
                if let Some(port_str) = line.strip_prefix("KROKI_BROWSER_WORKER_PORT=") {
                    return Ok(port_str.parse::<u16>()?);
                }
            }
            anyhow::bail!("Worker stream ended cleanly before port was reported.")
        };

        let port = match tokio::time::timeout(startup_timeout, find_port).await {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => anyhow::bail!("Failed to read worker port: {}", e),
            Err(_) => {
                let _ = child.start_kill();
                anyhow::bail!("Timeout waiting for browser worker to start.");
            }
        };

        // Consume the rest of stdout to prevent pipe buffer locking
        tokio::spawn(async move { while let Ok(Some(_)) = reader.next_line().await {} });

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        tracing::info!("Browser worker established on port {}", port);

        Ok(Self {
            port,
            client,
            worker_process: Mutex::new(Some(child)),
        })
    }

    /// Evaluate diagram code inside the pooled Playwright browser.
    pub async fn evaluate(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
    ) -> DiagramResult<Vec<u8>> {
        #[derive(Serialize)]
        struct Payload<'a> {
            #[serde(rename = "type")]
            typ: &'a str,
            source: &'a str,
            format: &'a str,
        }

        let payload = Payload {
            typ: diagram_type,
            source,
            format,
        };

        let url = format!("http://127.0.0.1:{}/evaluate", self.port);
        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                DiagramError::ProcessFailed(format!("Browser worker HTTP request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            tracing::error!("Browser worker returned error {}: {}", status, err_text);
            return Err(DiagramError::ProcessFailed(format!(
                "Browser worker returned error {}: {}",
                status, err_text
            )));
        }

        let text = resp.text().await.map_err(|e| {
            DiagramError::ProcessFailed(format!("Failed to read SVG text from browser: {}", e))
        })?;

        Ok(text.into_bytes())
    }

    /// Fetches the internal connection pool health from the generic-pool worker.
    pub async fn get_pool_health(&self) -> Result<serde_json::Value> {
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() {
            let json = resp.json::<serde_json::Value>().await?;
            Ok(json)
        } else {
            anyhow::bail!("Worker health check failed with status: {}", resp.status())
        }
    }
}
