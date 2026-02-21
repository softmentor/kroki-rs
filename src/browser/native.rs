use crate::browser::backend::BrowserBackend;
use crate::diagrams::{DiagramError, DiagramResult};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use headless_chrome::Browser;

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

        // 1. Prepare rendering environment
        let html = include_str!("../../resources/browser/index.html");

        // Use a data URI to load the basic harness
        let data_uri = format!("data:text/html;base64,{}", STANDARD.encode(html.as_bytes()));
        tab.navigate_to(&data_uri)
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
                // CheerpJ loader (CDN for now, or we could embed it too if we had it)
                let loader_script = "const s = document.createElement('script'); s.src = 'https://cjrtnc.leaningtech.com/2.0/loader.js'; document.head.appendChild(s);";
                tab.evaluate(loader_script, false)
                    .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

                // Wait for loader to be available
                let wait_js = "new Promise((resolve) => { const check = () => { if (typeof cheerpjInit !== 'undefined') resolve(); else setTimeout(check, 50); }; check(); })";
                tab.evaluate(wait_js, true)
                    .map_err(|e| DiagramError::ProcessFailed(format!("CheerpJ timeout: {}", e)))?;

                // Inject PlantUML JAR.js (it's 17MB, maybe we should read it from file instead of including at compile time?)
                // For this version, let's try to load it from the filesystem via the browser's view.
                // However, headless_chrome can't easily access local files due to security.
                // We'll use a direct injection for now to be safe, though it bloats the binary.
                // TODO: Optimization - move to dynamic loading from a specific kroki resource dir.
                let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let plantuml_path = manifest_dir.join("resources/plantuml/plantuml-core.jar.js");
                let plantuml_js = std::fs::read_to_string(&plantuml_path).map_err(|e| {
                    DiagramError::ProcessFailed(format!(
                        "Could not read plantuml-core at {}: {}",
                        plantuml_path.display(),
                        e
                    ))
                })?;

                // In CheerpJ 2.0, we just need to load the script
                tab.evaluate(&plantuml_js, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to load PlantUML core: {}", e))
                })?;
            }
            _ => {
                return Err(DiagramError::UnsupportedFormat {
                    provider: diagram_type.to_string(),
                    format: _format.to_string(),
                })
            }
        }

        // 3. Execute render
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
                DiagramError::ProcessFailed("Render returned null or non-string".to_string())
            })?;

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
