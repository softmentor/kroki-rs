use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use crate::utils::decode;
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
    let registry = DiagramRegistry::new(&capabilities);

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

    // 4. Generate
    match provider.generate(&source, &format) {
        Ok(bytes) => {
            let content_type = match format.as_str() {
                "svg" => "image/svg+xml",
                "png" => "image/png",
                "pdf" => "application/pdf",
                "txt" => "text/plain",
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
