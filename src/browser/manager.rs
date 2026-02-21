use crate::browser::backend::BrowserBackend;
use crate::browser::native::NativeBackend;
use crate::browser::playwright::PlaywrightBackend;
use crate::diagrams::DiagramResult;
use anyhow::Result;
use std::sync::Arc;

/// Unified manager for browser-based rendering backends.
/// Abstracts away the specific implementation (Native or Playwright).
#[derive(Clone)]
pub struct BrowserManager {
    backend: Arc<dyn BrowserBackend>,
}

impl BrowserManager {
    /// Launches the preferred browser backend.
    /// Prefers Native (headless_chrome) if available, falling back to Playwright if needed.
    pub async fn start(pool_size: usize, context_ttl: usize) -> Result<Self> {
        // 1. Try Native Backend (v0.0.5 target)
        match NativeBackend::new().await {
            Ok(backend) => {
                tracing::info!("Initialized native browser backend (headless_chrome)");
                return Ok(Self {
                    backend: Arc::new(backend),
                });
            }
            Err(e) => {
                tracing::warn!(
                    "Native browser backend failed: {}. Falling back to Playwright.",
                    e
                );
            }
        }

        // 2. Fallback to Playwright Backend (v0.0.4 legacy)
        let backend = PlaywrightBackend::start(pool_size, context_ttl).await?;
        tracing::info!("Initialized legacy browser backend (Playwright/Node.js)");
        Ok(Self {
            backend: Arc::new(backend),
        })
    }

    /// Evaluate diagram code inside the preferred browser backend.
    pub async fn evaluate(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
    ) -> DiagramResult<Vec<u8>> {
        self.backend.render(diagram_type, source, format).await
    }

    /// Fetches health information from the active backend.
    pub async fn get_pool_health(&self) -> Result<serde_json::Value> {
        Ok(self.backend.health().await)
    }
}
