use crate::browser::backend::BrowserBackend;
use crate::diagrams::{DiagramError, DiagramResult};
use async_trait::async_trait;
use headless_chrome::protocol::cdp::types::Event;
use headless_chrome::Browser;
use std::sync::Arc;

/// Native browser backend using the `headless_chrome` crate.
/// Eliminates the need for a separate Node.js worker process.
pub struct NativeBackend {
    browser: Browser,
}

impl NativeBackend {
    /// Initializes a new native browser instance.
    pub fn new() -> Result<Self, String> {
        use headless_chrome::LaunchOptions;

        // Use custom launch options for better compatibility (e.g. CI/containers)
        let options = LaunchOptions {
            args: vec![
                std::ffi::OsStr::new("--no-sandbox"),
                std::ffi::OsStr::new("--disable-setuid-sandbox"),
                std::ffi::OsStr::new("--disable-dev-shm-usage"),
                std::ffi::OsStr::new("--disable-gpu"),
                std::ffi::OsStr::new("--allow-file-access-from-files"),
            ],
            ..Default::default()
        };

        let browser = Browser::new(options).map_err(|e| e.to_string())?;
        Ok(Self { browser })
    }
}

#[async_trait]
impl BrowserBackend for NativeBackend {
    async fn render(
        &self,
        diagram_type: &str,
        source: &str,
        _format: &str,
    ) -> DiagramResult<Vec<u8>> {
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

        // Enable console log capturing to pipe diagnostics to Rust tracing
        tab.add_event_listener(Arc::new(move |event: &Event| {
            if let Event::RuntimeConsoleAPICalled(evt) = event {
                let args: Vec<String> = evt
                    .params
                    .args
                    .iter()
                    .filter_map(|p| {
                        p.value.as_ref().and_then(|v: &serde_json::Value| {
                            v.as_str().map(|s: &str| s.to_string())
                        })
                    })
                    .collect();
                tracing::debug!(
                    "Browser Console [{:?}]: {}",
                    evt.params.Type,
                    args.join(" ")
                );
            }
        }))
        .map_err(|e| {
            DiagramError::ProcessFailed(format!("Failed to enable console capture: {}", e))
        })?;

        // 1. Prepare rendering environment using a temporary file
        // Data URIs result in a 'null' origin which causes CheerpJ to fail.
        let html = include_str!("../../resources/browser/index.html");
        let mut temp_file = tempfile::Builder::new()
            .prefix("kroki-harness-")
            .suffix(".html")
            .tempfile()
            .map_err(|e| {
                DiagramError::ProcessFailed(format!("Failed to create temp harness: {}", e))
            })?;

        use std::io::Write;
        temp_file
            .write_all(html.as_bytes())
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

        let file_uri = format!("file://{}", temp_file.path().display());
        tracing::debug!("Navigating to harness: {}", file_uri);

        tab.navigate_to(&file_uri)
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

        // 2. Inject required libraries
        match diagram_type {
            "mermaid" => {
                let js = include_str!("../../resources/browser/mermaid.min.js");
                tab.evaluate(js, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to load Mermaid: {}", e))
                })?;
            }
            "bpmn" => {
                let js = include_str!("../../resources/browser/bpmn-viewer.production.min.js");
                tab.evaluate(js, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to load BPMN: {}", e))
                })?;
            }
            "plantuml" => {
                // CheerpJ loader (CDN for now)
                let loader_script = "const s = document.createElement('script'); s.src = 'https://cjrtnc.leaningtech.com/2.0/loader.js'; document.head.appendChild(s);";
                tab.evaluate(loader_script, false)
                    .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

                // Wait for loader to be available with timeout (10 seconds max)
                let wait_js = "new Promise((resolve, reject) => { const startTime = Date.now(); const check = () => { if (typeof cheerpjInit !== 'undefined') { resolve(true); } else if (Date.now() - startTime > 10000) { reject(new Error('CheerpJ loader timeout - CDN may be unreachable')); } else { setTimeout(check, 100); } }; check(); })";
                tab.evaluate(wait_js, true).map_err(|e| {
                    DiagramError::ProcessFailed(format!("CheerpJ initialization failed: {}", e))
                })?;
            }
            _ => {
                return Err(DiagramError::UnsupportedFormat {
                    provider: diagram_type.to_string(),
                    format: _format.to_string(),
                })
            }
        }

        // 3. For PlantUML, load the JAR.js lazily here (after CheerpJ is ready)
        if diagram_type == "plantuml" {
            let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let plantuml_path = manifest_dir.join("resources/plantuml/plantuml-core.jar.js");
            let plantuml_js = std::fs::read_to_string(&plantuml_path).map_err(|e| {
                DiagramError::ProcessFailed(format!(
                    "Could not read plantuml-core at {}: {}",
                    plantuml_path.display(),
                    e
                ))
            })?;

            tab.evaluate(&plantuml_js, false).map_err(|e| {
                DiagramError::ProcessFailed(format!("Failed to load PlantUML core: {}", e))
            })?;
        }

        // 4. Execute render
        let render_expr = match diagram_type {
            "mermaid" | "bpmn" | "plantuml" => format!(
                "window.kroki.render{}({})",
                match diagram_type {
                    "mermaid" => "Mermaid",
                    "bpmn" => "Bpmn",
                    "plantuml" => "PlantUml",
                    _ => unreachable!(),
                },
                serde_json::to_string(source).unwrap()
            ),
            _ => unreachable!(),
        };

        let remote_object = tab
            .evaluate(&render_expr, true)
            .map_err(|e| DiagramError::ProcessFailed(format!("Render execution failed: {}", e)))?;

        let result = remote_object
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| {
                // Try to get console errors and debug logs for better diagnostics
                let mut error_details = String::new();

                // Get stored error message
                if let Ok(error_obj) = tab.evaluate("window.kroki.lastError", false) {
                    if let Some(val) = error_obj
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                    {
                        error_details.push_str(&format!("Last error: {}\n", val));
                    }
                }

                // Get debug logs
                if let Ok(debug_obj) = tab.evaluate("JSON.stringify(window.kroki.debugLog)", false)
                {
                    if let Some(val) = debug_obj
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                    {
                        error_details.push_str(&format!("Debug log: {}\n", val));
                    }
                }

                let final_msg = if error_details.is_empty() {
                    "Render returned null or non-string (unable to get error details)".to_string()
                } else {
                    error_details
                };

                DiagramError::ProcessFailed(final_msg)
            })?;

        if result.is_empty() {
            let mut error_msg =
                "Render produced empty output - render function may have failed silently"
                    .to_string();

            if let Ok(debug_obj) = tab.evaluate("JSON.stringify(window.kroki.debugLog)", false) {
                if let Some(val) = debug_obj
                    .value
                    .as_ref()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                {
                    error_msg.push_str(&format!("\nDebug log: {}", val));
                }
            }

            return Err(DiagramError::ProcessFailed(error_msg));
        }

        Ok(result.into_bytes())
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
            "tabs": tabs_count
        })
    }
}
