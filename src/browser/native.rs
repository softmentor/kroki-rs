#[cfg(feature = "native-browser")]
mod native_impl {
    use crate::browser::backend::BrowserBackend;
    use crate::diagrams::{DiagramError, DiagramResult};
    use async_trait::async_trait;

    use headless_chrome::Browser;
    use std::io::Write;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    /// Native browser backend using the `headless_chrome` crate.
    pub struct NativeBackend {
        browser: Browser,
        harness_url: String,
        semaphore: Arc<Semaphore>,
        _harness_file: tempfile::NamedTempFile,
    }

    impl NativeBackend {
        pub async fn new(pool_size: usize) -> Result<Self, String> {
            let mut temp_file = tempfile::Builder::new()
                .suffix(".html")
                .tempfile()
                .map_err(|e| format!("Failed to create temp harness: {}", e))?;

            let mermaid_js = include_str!("../../resources/browser/mermaid.min.js");
            let bpmn_js = include_str!("../../resources/browser/bpmn-viewer.production.min.js");
            let index_html = include_str!("../../resources/browser/index.html");

            // Inject scripts into the HTML
            let html = index_html.replace(
                "<!-- KROKI_SCRIPTS -->",
                &("<script>".to_string()
                    + mermaid_js
                    + "</script><script>"
                    + bpmn_js
                    + "</script>"),
            );

            temp_file
                .write_all(html.as_bytes())
                .map_err(|e| format!("Failed to write harness: {}", e))?;

            let harness_url = format!("file://{}", temp_file.path().to_str().unwrap());
            tracing::debug!("Local serverless harness created at {}", harness_url);

            use headless_chrome::LaunchOptions;
            let options = LaunchOptions {
                args: vec![
                    std::ffi::OsStr::new("--no-sandbox"),
                    std::ffi::OsStr::new("--disable-setuid-sandbox"),
                    std::ffi::OsStr::new("--disable-dev-shm-usage"),
                    std::ffi::OsStr::new("--disable-gpu"),
                    std::ffi::OsStr::new("--disable-web-security"),
                    std::ffi::OsStr::new("--disable-software-rasterizer"),
                    std::ffi::OsStr::new("--disable-features=IsolateOrigins,site-per-process"),
                    std::ffi::OsStr::new("--font-render-hinting=none"),
                    std::ffi::OsStr::new("--allow-file-access-from-files"),
                ],
                idle_browser_timeout: std::time::Duration::from_secs(120),
                ..Default::default()
            };

            let browser = tokio::task::spawn_blocking(move || Browser::new(options))
                .await
                .map_err(|e| format!("Browser spawn join failed: {}", e))?
                .map_err(|e| e.to_string())?;
            let semaphore = Arc::new(Semaphore::new(pool_size));

            Ok(Self {
                browser,
                harness_url,
                semaphore,
                _harness_file: temp_file,
            })
        }

        async fn do_render(
            &self,
            tab: &headless_chrome::Tab,
            diagram_type: &str,
            source: &str,
            _format: &str,
        ) -> DiagramResult<Vec<u8>> {
            tab.navigate_to(&self.harness_url)
                .map_err(|e| DiagramError::ProcessFailed(format!("Navigation failed: {}", e)))?;
            tab.wait_for_element("#container")
                .map_err(|e| DiagramError::ProcessFailed(format!("Harness load timeout: {}", e)))?;

            let font_injection = "const style = document.getElementById('kroki-fonts'); if (style) { style.innerHTML = window.krokiFontCss || ''; }";
            tab.evaluate(font_injection, false).map_err(|e| {
                DiagramError::ProcessFailed(format!("Font injection failed: {}", e))
            })?;

            match diagram_type {
                "mermaid" => {
                    tab.evaluate("new Promise(r => { const check = () => window.mermaid ? r() : setTimeout(check, 50); check(); })", true)
                        .map_err(|e| DiagramError::ProcessFailed(format!("Mermaid load timeout: {}", e)))?;
                }
                "bpmn" => {
                    tab.evaluate("new Promise(r => { const check = () => window.BpmnJS ? r() : setTimeout(check, 50); check(); })", true)
                        .map_err(|e| DiagramError::ProcessFailed(format!("BPMN load timeout: {}", e)))?;
                }
                _ => {
                    return Err(DiagramError::UnsupportedFormat {
                        provider: diagram_type.to_string(),
                        format: _format.to_string(),
                    })
                }
            }

            let render_expr = format!(
                "window.kroki.render{}({})",
                match diagram_type {
                    "mermaid" => "Mermaid",
                    "bpmn" => "Bpmn",
                    _ => unreachable!(),
                },
                serde_json::to_string(source).unwrap()
            );

            let remote_object = tab.evaluate(&render_expr, true).map_err(|e| {
                DiagramError::ProcessFailed(format!("Render execution failed: {}", e))
            })?;

            let result = remote_object
                .value
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or_else(|| {
                    DiagramError::ProcessFailed("Render returned null or non-string".to_string())
                })?;

            if result.is_empty() {
                return Err(DiagramError::ProcessFailed(
                    "Render produced empty output".to_string(),
                ));
            }

            Ok(result.into_bytes())
        }
    }

    #[async_trait]
    impl BrowserBackend for NativeBackend {
        async fn render(
            &self,
            diagram_type: &str,
            source: &str,
            format: &str,
        ) -> DiagramResult<Vec<u8>> {
            let _permit = self.semaphore.acquire().await.map_err(|_| {
                DiagramError::ProcessFailed("Native backend semaphore was closed".to_string())
            })?;

            tracing::debug!("Creating new tab...");
            let tab = self
                .browser
                .new_tab()
                .map_err(|e| DiagramError::ProcessFailed(format!("Failed to create tab: {}", e)))?;

            let result = self.do_render(&tab, diagram_type, source, format).await;

            // Ensure tab is closed regardless of success
            let _ = tab.close(false);

            result
        }

        async fn health(&self) -> serde_json::Value {
            let tabs_count = if let Ok(lock) = self.browser.get_tabs().lock() {
                lock.len()
            } else {
                0
            };

            serde_json::json!({
                "status": "ok",
                "backend": "headless_chrome",
                "tabs": tabs_count,
                "harness_url": self.harness_url,
                "concurrency_permits_available": self.semaphore.available_permits()
            })
        }
    }

    pub use NativeBackend as Backend;
}

#[cfg(feature = "native-browser")]
pub use native_impl::Backend as NativeBackend;
