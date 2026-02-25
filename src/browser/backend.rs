use crate::diagrams::DiagramResult;
use async_trait::async_trait;

/// Abstract interface for browser-based diagram rendering.
/// Managed via a persistent pool of browser contexts.
#[async_trait]
pub trait BrowserBackend: Send + Sync {
    /// Renders a diagram using the browser.
    async fn render(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
    ) -> DiagramResult<Vec<u8>>;

    /// Returns health information about the browser instance/pool.
    async fn health(&self) -> serde_json::Value;
}
