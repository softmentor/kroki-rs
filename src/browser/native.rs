use crate::browser::backend::BrowserBackend;
use crate::diagrams::{DiagramError, DiagramResult};
use async_trait::async_trait;
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use headless_chrome::protocol::cdp::types::Event;
use headless_chrome::Browser;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// Native browser backend using the `headless_chrome` crate.
/// Eliminates the need for a separate Node.js worker process.
pub struct NativeBackend {
    browser: Browser,
    harness_url: String,
    _shutdown_tx: oneshot::Sender<()>,
}

impl NativeBackend {
    /// Initializes a new native browser instance and starts a local harness server.
    pub async fn new() -> Result<Self, String> {
        // 1. Start local harness server to provide a valid HTTP origin for CheerpJ
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (port_tx, port_rx) = oneshot::channel();

        tokio::spawn(async move {
            let app = Router::new()
                .route("/", get(handle_index))
                .route("/js/{name}", get(handle_js));

            let listener = match TcpListener::bind("127.0.0.1:0").await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Failed to bind harness server: {}", e);
                    return;
                }
            };
            let port = listener.local_addr().unwrap().port();
            let _ = port_tx.send(port);

            let server = axum::serve(listener, app);
            let graceful = server.with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });

            if let Err(e) = graceful.await {
                tracing::error!("Harness server error: {}", e);
            }
        });

        // Wait for server to bind
        let port = match port_rx.await {
            Ok(p) => p,
            Err(_) => return Err("Failed to start local harness server".to_string()),
        };
        let harness_url = format!("http://localhost:{}", port);
        tracing::debug!("Local harness server started at {}", harness_url);

        // 2. Initialize browser
        use headless_chrome::LaunchOptions;
        let options = LaunchOptions {
            args: vec![
                std::ffi::OsStr::new("--no-sandbox"),
                std::ffi::OsStr::new("--disable-setuid-sandbox"),
                std::ffi::OsStr::new("--disable-dev-shm-usage"),
                std::ffi::OsStr::new("--disable-gpu"),
                std::ffi::OsStr::new("--disable-web-security"),
            ],
            ..Default::default()
        };

        let browser = Browser::new(options).map_err(|e| e.to_string())?;
        Ok(Self {
            browser,
            harness_url,
            _shutdown_tx: shutdown_tx,
        })
    }
}

async fn handle_index() -> impl IntoResponse {
    let html = include_str!("../../resources/browser/index.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

async fn handle_js(Path(name): Path<String>) -> impl IntoResponse {
    let (content, content_type) = match name.as_str() {
        "mermaid.min.js" => (
            include_str!("../../resources/browser/mermaid.min.js").as_bytes(),
            "application/javascript",
        ),
        "bpmn-viewer.production.min.js" => (
            include_str!("../../resources/browser/bpmn-viewer.production.min.js").as_bytes(),
            "application/javascript",
        ),
        "plantuml-core.jar.js" => {
            // This is large (17MB), but include_bytes! handles it fine
            let bytes = include_bytes!("../../resources/plantuml/plantuml-core.jar.js");
            (bytes.as_slice(), "application/javascript")
        }
        _ => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
    };

    ([(header::CONTENT_TYPE, content_type)], content).into_response()
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

        // Enable console log capturing
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

        // 1. Prepare rendering environment using local HTTP server
        tab.navigate_to(&self.harness_url)
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

        // 2. Inject required libraries via script tags (cleaner and faster)
        match diagram_type {
            "mermaid" => {
                let js = "const s = document.createElement('script'); s.src = '/js/mermaid.min.js'; document.head.appendChild(s);";
                tab.evaluate(js, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to load Mermaid: {}", e))
                })?;
                // Wait for library
                tab.evaluate("new Promise(r => { const check = () => window.mermaid ? r() : setTimeout(check, 50); check(); })", true)
                    .map_err(|e| DiagramError::ProcessFailed(format!("Mermaid load timeout: {}", e)))?;
            }
            "bpmn" => {
                let js = "const s = document.createElement('script'); s.src = '/js/bpmn-viewer.production.min.js'; document.head.appendChild(s);";
                tab.evaluate(js, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to load BPMN: {}", e))
                })?;
                // Wait for library
                tab.evaluate("new Promise(r => { const check = () => window.BpmnJS ? r() : setTimeout(check, 50); check(); })", true)
                    .map_err(|e| DiagramError::ProcessFailed(format!("BPMN load timeout: {}", e)))?;
            }
            "plantuml" => {
                // CheerpJ loader (CDN for now, or we could self-host the loader.js too if needed)
                let loader_script = "const s = document.createElement('script'); s.src = 'https://cjrtnc.leaningtech.com/2.0/loader.js'; document.head.appendChild(s);";
                tab.evaluate(loader_script, false)
                    .map_err(|e| DiagramError::ProcessFailed(e.to_string()))?;

                // Wait for loader
                let wait_js = "new Promise((resolve, reject) => { const startTime = Date.now(); const check = () => { if (typeof cheerpjInit !== 'undefined') { resolve(true); } else if (Date.now() - startTime > 10000) { reject(new Error('CheerpJ loader timeout')); } else { setTimeout(check, 100); } }; check(); })";
                tab.evaluate(wait_js, true).map_err(|e| {
                    DiagramError::ProcessFailed(format!("CheerpJ initialization failed: {}", e))
                })?;

                // Self-host the huge core JAR JS
                let jar_script = "const s = document.createElement('script'); s.src = '/js/plantuml-core.jar.js'; document.head.appendChild(s);";
                tab.evaluate(jar_script, false).map_err(|e| {
                    DiagramError::ProcessFailed(format!("Failed to inject PlantUML core: {}", e))
                })?;
            }
            _ => {
                return Err(DiagramError::UnsupportedFormat {
                    provider: diagram_type.to_string(),
                    format: _format.to_string(),
                })
            }
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
                let mut error_details = String::new();
                if let Ok(error_obj) = tab.evaluate("window.kroki.lastError", false) {
                    if let Some(val) = error_obj
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                    {
                        error_details.push_str(&format!("Last error: {}\n", val));
                    }
                }
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
                    "Render returned null or non-string".to_string()
                } else {
                    error_details
                };
                DiagramError::ProcessFailed(final_msg)
            })?;

        if result.is_empty() {
            return Err(DiagramError::ProcessFailed(
                "Render produced empty output".to_string(),
            ));
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
            "tabs": tabs_count,
            "harness_url": self.harness_url
        })
    }
}
