use crate::browser::backend::BrowserBackend;
#[cfg(feature = "native-browser")]
use crate::browser::native::NativeBackend;
use crate::diagrams::DiagramResult;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// Unified manager for browser-based rendering backends.
/// Abstracts away the specific implementation (Native).
#[derive(Clone)]
pub struct BrowserManager {
    backend: Arc<dyn BrowserBackend>,
}

impl BrowserManager {
    /// Launches the preferred browser backend.
    /// Prefers Native (headless_chrome) if available.
    pub async fn start(_pool_size: usize, _context_ttl: usize) -> Result<Self> {
        #[cfg(feature = "native-browser")]
        {
            match NativeBackend::new(_pool_size).await {
                Ok(backend) => {
                    tracing::info!("Initialized native browser backend (headless_chrome)");
                    Ok(Self {
                        backend: Arc::new(backend),
                    })
                }
                Err(e) => Err(anyhow!("Native browser backend failed to start: {}", e)),
            }
        }

        #[cfg(not(feature = "native-browser"))]
        {
            Err(anyhow!("Browser-based rendering is disabled in this build. Rebuild with --features native-browser."))
        }
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
