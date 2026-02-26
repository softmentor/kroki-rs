#[cfg(feature = "native-browser")]
mod native_impl {
    use crate::browser::backend::BrowserBackend;
    use crate::diagrams::{DiagramError, DiagramResult};
    use async_trait::async_trait;
    use headless_chrome::{Browser, LaunchOptions};
    use std::ffi::OsStr;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::{Builder, NamedTempFile};
    use tokio::sync::{RwLock, Semaphore};

    const CHROME_ARGS: &[&str] = &[
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
        "--disable-gpu",
        "--disable-web-security",
        "--disable-software-rasterizer",
        "--disable-features=IsolateOrigins,site-per-process",
        "--font-render-hinting=none",
        "--allow-file-access-from-files",
    ];

    const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(120);

    /// Native browser backend using the `headless_chrome` crate.
    pub struct NativeBackend {
        browser: Arc<RwLock<Browser>>,
        harness_url: String,
        semaphore: Arc<Semaphore>,
        _harness_file: NamedTempFile,
        context_ttl_requests: usize,
        request_count: AtomicUsize,
        restarting: AtomicBool,
    }

    impl NativeBackend {
        pub async fn new(pool_size: usize, context_ttl_requests: usize) -> Result<Self, String> {
            let (harness_url, harness_file) = Self::build_harness()?;
            let browser = Self::spawn_browser().await?;
            Ok(Self {
                browser: Arc::new(RwLock::new(browser)),
                harness_url,
                semaphore: Arc::new(Semaphore::new(pool_size)),
                _harness_file: harness_file,
                context_ttl_requests,
                request_count: AtomicUsize::new(0),
                restarting: AtomicBool::new(false),
            })
        }

        fn build_harness() -> Result<(String, NamedTempFile), String> {
            let mut temp_file = Builder::new()
                .suffix(".html")
                .tempfile()
                .map_err(|e| format!("Failed to create temp harness: {}", e))?;

            let mermaid_js = include_str!("../../resources/browser/mermaid.min.js");
            let bpmn_js = include_str!("../../resources/browser/bpmn-viewer.production.min.js");
            let index_html = include_str!("../../resources/browser/index.html");

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

            let path = temp_file
                .path()
                .to_str()
                .ok_or_else(|| "Failed to build harness URL".to_string())?;
            let harness_url = format!("file://{}", path);
            tracing::debug!("Local serverless harness created at {}", harness_url);

            Ok((harness_url, temp_file))
        }

        fn default_launch_options() -> LaunchOptions<'static> {
            let args: Vec<&'static OsStr> = CHROME_ARGS.iter().map(OsStr::new).collect();

            LaunchOptions {
                args,
                idle_browser_timeout: DEFAULT_IDLE_TIMEOUT,
                ..Default::default()
            }
        }

        async fn spawn_browser() -> Result<Browser, String> {
            let options = Self::default_launch_options();
            tokio::task::spawn_blocking(move || Browser::new(options))
                .await
                .map_err(|e| format!("Browser spawn join failed: {}", e))?
                .map_err(|e| e.to_string())
        }

        async fn restart_browser(&self) -> Result<(), String> {
            let new_browser = Self::spawn_browser().await?;
            let mut guard = self.browser.write().await;
            *guard = new_browser;
            Ok(())
        }

        fn should_restart(&self) -> bool {
            if self.context_ttl_requests == 0 {
                return false;
            }
            let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;
            count >= self.context_ttl_requests
        }

        async fn maybe_restart(&self) {
            if !self.should_restart() {
                return;
            }
            if self
                .restarting
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                return;
            }

            if let Err(err) = self.restart_browser().await {
                tracing::error!("Native browser restart failed: {}", err);
            }

            self.request_count.store(0, Ordering::Relaxed);
            self.restarting.store(false, Ordering::Release);
        }

        async fn acquire_browser(&self) -> Browser {
            let guard = self.browser.read().await;
            guard.clone()
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

            let font_injection =
                "const style = document.getElementById('kroki-fonts'); if (style) { style.innerHTML = window.krokiFontCss || ''; }";
            tab.evaluate(font_injection, false).map_err(|e| {
                DiagramError::ProcessFailed(format!("Font injection failed: {}", e))
            })?;

            match diagram_type {
                "mermaid" => {
                    tab.evaluate(
                        "new Promise(r => { const check = () => window.mermaid ? r() : setTimeout(check, 50); check(); })",
                        true,
                    )
                    .map_err(|e| DiagramError::ProcessFailed(format!("Mermaid load timeout: {}", e)))?;
                }
                "bpmn" => {
                    tab.evaluate(
                        "new Promise(r => { const check = () => window.BpmnJS ? r() : setTimeout(check, 50); check(); })",
                        true,
                    )
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
            self.maybe_restart().await;

            let _permit = self.semaphore.acquire().await.map_err(|_| {
                DiagramError::ProcessFailed("Native backend semaphore was closed".to_string())
            })?;

            tracing::debug!("Creating new tab...");
            let browser = self.acquire_browser().await;
            let tab = browser
                .new_tab()
                .map_err(|e| DiagramError::ProcessFailed(format!("Failed to create tab: {}", e)))?;

            let result = self.do_render(&tab, diagram_type, source, format).await;

            let _ = tab.close(false);

            result
        }

        async fn health(&self) -> serde_json::Value {
            let browser = self.acquire_browser().await;
            let tabs_count = browser
                .get_tabs()
                .lock()
                .map(|tabs| tabs.len())
                .unwrap_or(0);

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
