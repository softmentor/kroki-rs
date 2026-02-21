use crate::config::SUPPORTED_FORMATS;
use crate::server::AppState;
use crate::utils::decode;
use crate::utils::image_converter;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use axum::response::Html;

/// Root discovery page handler.
pub async fn root(State(state): State<AppState>) -> impl IntoResponse {
    let capabilities = state.registry.known_types();
    let admin_port = state.config.server.admin_port;

    let auth_status = if state.config.server.auth.enabled {
        "Enabled"
    } else {
        "Disabled (Dev Mode)"
    };
    let rate_limit_status = if state.config.server.rate_limit.enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    let metrics_status = if state.config.server.metrics.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kroki-rs | Discovery</title>
    <style>
        :root {{
            --primary: #6366f1;
            --primary-dark: #4f46e5;
            --bg: #0f172a;
            --card-bg: #1e293b;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --success: #22c55e;
            --warning: #eab308;
        }}
        body {{
            font-family: 'Inter', -apple-system, sans-serif;
            background-color: var(--bg);
            color: var(--text);
            margin: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            min-height: 100vh;
            padding: 2rem;
        }}
        .container {{
            max-width: 900px;
            width: 100%;
        }}
        header {{
            text-align: center;
            margin-bottom: 3rem;
        }}
        h1 {{
            font-size: 3rem;
            margin: 0;
            background: linear-gradient(to right, #818cf8, #c084fc);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .version {{
            font-size: 0.875rem;
            color: var(--text-muted);
            margin-top: 0.5rem;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }}
        .card {{
            background: var(--card-bg);
            border-radius: 1rem;
            padding: 1.5rem;
            border: 1px solid #334155;
            transition: transform 0.2s, border-color 0.2s;
        }}
        .card:hover {{
            transform: translateY(-4px);
            border-color: var(--primary);
        }}
        .card h2 {{
            margin-top: 0;
            font-size: 1.25rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}
        .status-pill {{
            font-size: 0.75rem;
            padding: 0.25rem 0.625rem;
            border-radius: 9999px;
            font-weight: 600;
            text-transform: uppercase;
        }}
        .status-enabled {{ background: #064e3b; color: #6ee7b7; }}
        .status-disabled {{ background: #450a0a; color: #fca5a5; }}
        
        .endpoints {{
            list-style: none;
            padding: 0;
            margin: 1.5rem 0 0;
        }}
        .endpoints li {{
            margin-bottom: 0.75rem;
        }}
        .endpoints a {{
            color: var(--primary);
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            font-weight: 500;
        }}
        .endpoints a:hover {{
            text-decoration: underline;
        }}
        
        .showcase {{
            background: var(--card-bg);
            border-radius: 1rem;
            padding: 2rem;
            border: 1px solid #334155;
        }}
        .showcase h2 {{ margin-top: 0; }}
        .provider-list {{
            display: flex;
            flex-wrap: wrap;
            gap: 0.75rem;
            margin-top: 1.5rem;
        }}
        .provider-tag {{
            background: #334155;
            padding: 0.5rem 1rem;
            border-radius: 0.5rem;
            font-size: 0.875rem;
            font-weight: 500;
        }}
        .footer {{
            margin-top: auto;
            padding-top: 3rem;
            color: var(--text-muted);
            font-size: 0.875rem;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Kroki-rs</h1>
            <div class="version">v{} Discovery Service</div>
        </header>

        <div class="grid">
            <div class="card">
                <h2>Service Status</h2>
                <div style="display: flex; flex-direction: column; gap: 0.75rem; margin-top: 1rem;">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span>Auth</span>
                        <span class="status-pill {}">{}</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span>Rate Limit</span>
                        <span class="status-pill {}">{}</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span>Metrics</span>
                        <span class="status-pill {}">{}</span>
                    </div>
                </div>
            </div>

            <div class="card">
                <h2>Endpoints</h2>
                <ul class="endpoints">
                    <li><a href="http://localhost:{}/health">🔍 Health Check</a></li>
                    {}
                    <li><a href="http://localhost:{}">⚙️ Admin Dashboard</a></li>
                </ul>
            </div>
        </div>

        <div class="showcase">
            <h2>Available Providers ({} registered)</h2>
            <div class="provider-list">
                {}
            </div>
        </div>

        <div class="footer">
            Built with Rust & Axum • <a href="https://github.com/softmentor/kroki-rs" style="color: inherit;">GitHub</a>
        </div>
    </div>
</body>
</html>
"#,
        env!("CARGO_PKG_VERSION"),
        if state.config.server.auth.enabled {
            "status-enabled"
        } else {
            "status-disabled"
        },
        auth_status,
        if state.config.server.rate_limit.enabled {
            "status-enabled"
        } else {
            "status-disabled"
        },
        rate_limit_status,
        if state.config.server.metrics.enabled {
            "status-enabled"
        } else {
            "status-disabled"
        },
        metrics_status,
        admin_port, // Health is on admin port
        if state.config.server.metrics.enabled && state.config.server.metrics.export_endpoint {
            {
                format!(
                    r#"<li><a href="http://localhost:{}/metrics">📊 Prometheus Metrics</a></li>"#,
                    admin_port
                )
            }
        } else {
            {
                "".to_string()
            }
        },
        admin_port,
        capabilities.len(),
        capabilities
            .iter()
            .map(|c| format!(r#"<div class="provider-tag">{}</div>"#, c))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Html(html)
}

/// Handler for retrieving diagrams via the Kroki GET API.
pub async fn get_diagram(
    Path((type_, format, source_encoded)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    tracing::info!("Request: type={}, format={}", type_, format);

    // 0. Initial metrics
    if state.config.server.metrics.enabled {
        crate::server::metrics::Metrics::increment_requests(&type_, &format);
    }

    // 1. Validate format against whitelist (TD-21)
    if !SUPPORTED_FORMATS.contains(&format.as_str()) {
        if state.config.server.metrics.enabled {
            crate::server::metrics::Metrics::increment_errors(
                &type_,
                &format,
                "unsupported_format",
            );
        }
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Unsupported format '{}'. Supported: {}",
                format,
                SUPPORTED_FORMATS.join(", ")
            ),
        )
            .into_response();
    }

    // 2. Decode payload
    let source = match decode(&source_encoded) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to decode source: {}", e);
            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::increment_errors(&type_, &format, "decode_error");
            }
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to decode source: {}", e),
            )
                .into_response();
        }
    };

    if state.config.server.metrics.enabled {
        crate::server::metrics::Metrics::record_payload_size(&type_, &format, source.len() as f64);
    }

    // 3. Validate input size (TD-19)
    if source.len() > state.config.server.max_input_size {
        if state.config.server.metrics.enabled {
            crate::server::metrics::Metrics::increment_errors(&type_, &format, "payload_too_large");
        }
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Input too large ({} bytes). Maximum allowed: {} bytes",
                source.len(),
                state.config.server.max_input_size
            ),
        )
            .into_response();
    }

    // 4. Find provider from pre-built registry (TD-04)
    let provider = match state.registry.get(&type_) {
        Some(p) => p,
        None => {
            let known = state.registry.known_types();
            let msg = if known.is_empty() {
                "No diagram tools are available on this server".to_string()
            } else {
                format!(
                    "Diagram type '{}' is not available. Supported types: {}",
                    type_,
                    known.join(", ")
                )
            };
            tracing::warn!("{}", msg);
            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::increment_errors(
                    &type_,
                    &format,
                    "provider_not_found",
                );
            }
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
    };

    // 5. Check circuit breaker for this provider type
    if let Some(ref cb) = state.circuit_breaker {
        if !cb.should_allow(&type_) {
            tracing::warn!(
                "Circuit breaker OPEN for provider '{}' — rejecting request",
                type_
            );
            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::increment_errors(
                    &type_,
                    &format,
                    "circuit_breaker_open",
                );
                crate::server::metrics::Metrics::set_circuit_breaker_state(&type_, 1.0);
                // 1 = Open
            }
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(
                    "Provider '{}' is temporarily unavailable due to repeated failures. Please retry later.",
                    type_
                ),
            )
                .into_response();
        }
    }

    let is_webp = format.to_lowercase() == "webp";
    let base_format = if is_webp {
        if type_.to_lowercase() == "ditaa" {
            "png"
        } else {
            "svg"
        }
    } else {
        &format
    };

    // 6. Generate
    let render_start = std::time::Instant::now();
    match provider.generate(&source, base_format).await {
        Ok(mut bytes) => {
            let render_duration = render_start.elapsed().as_secs_f64();
            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::record_conversion_time(
                    &type_,
                    &format,
                    render_duration,
                );
            }

            // Record success for circuit breaker
            if let Some(ref cb) = state.circuit_breaker {
                cb.record_success(&type_);
                if state.config.server.metrics.enabled {
                    crate::server::metrics::Metrics::set_circuit_breaker_state(&type_, 0.0);
                    // 0 = Closed
                }
            }

            // Output size validation (TD-20)
            if bytes.len() > state.config.server.max_output_size {
                tracing::error!(
                    "Output too large ({} bytes, max: {} bytes) for type={}",
                    bytes.len(),
                    state.config.server.max_output_size,
                    type_
                );
                if state.config.server.metrics.enabled {
                    crate::server::metrics::Metrics::increment_errors(
                        &type_,
                        &format,
                        "output_too_large",
                    );
                }
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "Generated output exceeds size limit ({} bytes). Consider simplifying the diagram.",
                        bytes.len()
                    ),
                )
                    .into_response();
            }

            if is_webp {
                let fonts = state.config.all_fonts();

                let convert_result = if base_format == "png" {
                    image_converter::png_to_webp(&bytes, image_converter::WebpQuality::Lossless)
                        .await
                } else {
                    let cache_dir = crate::config::Config::resolve_cache_dir(None);
                    image_converter::svg_to_webp(
                        &bytes,
                        image_converter::WebpQuality::Lossless,
                        &fonts,
                        cache_dir.as_deref(),
                    )
                    .await
                };

                match convert_result {
                    Ok(webp_bytes) => {
                        bytes = webp_bytes;
                    }
                    Err(e) => {
                        tracing::error!("WebP conversion failed for {}: {}", type_, e);
                        if state.config.server.metrics.enabled {
                            crate::server::metrics::Metrics::increment_errors(
                                &type_,
                                &format,
                                "webp_conversion_error",
                            );
                        }
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Diagram generation failed. Check server logs for details.".to_string(),
                        )
                            .into_response();
                    }
                }
            }

            let content_type = match format.as_str() {
                "svg" => "image/svg+xml",
                "png" => "image/png",
                "pdf" => "application/pdf",
                "txt" => "text/plain",
                "webp" => "image/webp",
                _ => "application/octet-stream",
            };

            let total_duration = start_time.elapsed().as_secs_f64();
            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::record_duration(&type_, &format, total_duration);
            }

            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                bytes,
            )
                .into_response()
        }
        Err(e) => {
            // Record failure for circuit breaker
            if let Some(ref cb) = state.circuit_breaker {
                cb.record_failure(&type_);
                if state.config.server.metrics.enabled {
                    crate::server::metrics::Metrics::set_circuit_breaker_state(&type_, 1.0);
                    // Simplified state tracking
                }
            }

            let error_kind = match &e {
                crate::diagrams::DiagramError::ValidationFailed(_) => "validation_failed",
                crate::diagrams::DiagramError::UnsupportedFormat { .. } => "unsupported_format",
                crate::diagrams::DiagramError::ToolNotFound(_) => "tool_not_found",
                crate::diagrams::DiagramError::ExecutionTimeout { .. } => "timeout",
                _ => "internal_error",
            };

            if state.config.server.metrics.enabled {
                crate::server::metrics::Metrics::increment_errors(&type_, &format, error_kind);
            }

            tracing::error!("Generation failed for {}: {}", type_, e);
            let (status, msg) = match e {
                crate::diagrams::DiagramError::ValidationFailed(msg) => {
                    (StatusCode::BAD_REQUEST, msg)
                }
                crate::diagrams::DiagramError::UnsupportedFormat { .. } => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                crate::diagrams::DiagramError::ToolNotFound(_) => {
                    (StatusCode::SERVICE_UNAVAILABLE, e.to_string())
                }
                crate::diagrams::DiagramError::ExecutionTimeout { .. } => {
                    (StatusCode::GATEWAY_TIMEOUT, e.to_string())
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Diagram generation failed. Check server logs for details.".to_string(),
                ),
            };
            (status, msg).into_response()
        }
    }
}
