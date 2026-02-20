use crate::browser::BrowserManager;
use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MermaidProvider {
    browser: Arc<BrowserManager>,
    _timeout_ms: Option<u64>,
}

impl MermaidProvider {
    pub fn new(browser: Arc<BrowserManager>, timeout_ms: Option<u64>) -> Self {
        Self {
            browser,
            _timeout_ms: timeout_ms,
        }
    }
}

#[async_trait]
impl DiagramProvider for MermaidProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        self.browser.evaluate("mermaid", source, format).await
    }
}
