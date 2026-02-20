use crate::config::SUPPORTED_FORMATS;
use crate::server::AppState;
use crate::utils::decode;
use crate::utils::image_converter;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

/// Root handler returning a simple welcome message.
pub async fn root() -> &'static str {
    "Kroki-rs is running!"
}

/// Handler for retrieving diagrams via the Kroki GET API.
pub async fn get_diagram(
    Path((type_, format, source_encoded)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("Request: type={}, format={}", type_, format);

    // 1. Validate format against whitelist (TD-21)
    if !SUPPORTED_FORMATS.contains(&format.as_str()) {
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
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to decode source: {}", e),
            )
                .into_response();
        }
    };

    // 3. Validate input size (TD-19)
    if source.len() > state.config.server.max_input_size {
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
            return (StatusCode::NOT_FOUND, msg).into_response();
        }
    };

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

    // 5. Generate
    match provider.generate(&source, base_format).await {
        Ok(mut bytes) => {
            // Output size validation (TD-20)
            if bytes.len() > state.config.server.max_output_size {
                tracing::error!(
                    "Output too large ({} bytes, max: {} bytes) for type={}",
                    bytes.len(),
                    state.config.server.max_output_size,
                    type_
                );
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

            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                bytes,
            )
                .into_response()
        }
        Err(e) => {
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
