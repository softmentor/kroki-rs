use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
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
///
/// Path parameters:
/// - `type_`: The diagram type (e.g., "mermaid").
/// - `format`: The desired output format (e.g., "svg").
/// - `source_encoded`: The Base64URL encoded and compressed diagram source.
pub async fn get_diagram(
    Path((type_, format, source_encoded)): Path<(String, String, String)>,
    State(config): State<Config>,
) -> impl IntoResponse {
    tracing::info!("Request: type={}, format={}", type_, format);
    // 1. Decode payload
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

    // 2. Discover capabilities
    let capabilities = Capabilities::discover(&config);
    let registry = DiagramRegistry::new(&capabilities, &config);

    // 3. Find provider
    let provider = match registry.get(&type_) {
        Some(p) => p,
        None => {
            tracing::warn!("Diagram type '{}' not supported or tool not found", type_);
            return (
                StatusCode::NOT_FOUND,
                format!("Diagram type '{}' not supported or tool not found", type_),
            )
                .into_response();
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

    // 4. Generate
    match provider.generate(&source, base_format).await {
        Ok(mut bytes) => {
            if is_webp {
                // Fallback to empty fonts slice if the tool isn't specifically defined, or try to extract from tool config
                // Wait, we can just aggregate all fonts for simplicity or use a generic list. Let's extract from the specific tool config.
                let mut fonts = Vec::new();
                // Simple hack: grab all fonts from all tools for the server context since we evaluate them anyway
                fonts.extend_from_slice(&config.mermaid.fonts);
                fonts.extend_from_slice(&config.graphviz.fonts);
                fonts.extend_from_slice(&config.plantuml.fonts);
                fonts.extend_from_slice(&config.excalidraw.fonts);

                let convert_result = if base_format == "png" {
                    image_converter::png_to_webp(&bytes, image_converter::WebpQuality::Lossless)
                        .await
                } else {
                    image_converter::svg_to_webp(
                        &bytes,
                        image_converter::WebpQuality::Lossless,
                        &fonts,
                        None,
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
                            format!("WebP conversion failed: {}", e),
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Generation failed: {}", e),
            )
                .into_response()
        }
    }
}
