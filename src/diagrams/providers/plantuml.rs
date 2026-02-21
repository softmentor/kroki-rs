use crate::browser::BrowserManager;
use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::sync::Arc;

pub struct PlantUmlProvider {
    browser: Arc<BrowserManager>,
    _timeout_ms: Option<u64>,
}

impl PlantUmlProvider {
    pub fn new(browser: Arc<BrowserManager>, timeout_ms: Option<u64>) -> Self {
        Self {
            browser,
            _timeout_ms: timeout_ms,
        }
    }
}

#[async_trait]
impl DiagramProvider for PlantUmlProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        // Redirect to browser-based rendering using CheerpJ + plantuml-core.jar.js
        self.browser.evaluate("plantuml", source, format).await
    }
}
