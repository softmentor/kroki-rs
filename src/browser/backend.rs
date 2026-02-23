use crate::diagrams::DiagramResult;
use async_trait::async_trait;

/// Abstract interface for browser-based diagram rendering.
/// Allows swapping between Playwright (Node.js) and native Rust backends (headless_chrome).
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
