use serde::{Deserialize, Serialize};

/// DTO for a diagram rendering request.
/// Decouples external input (HTTP/CLI) from the internal domain models.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderRequestDto {
    /// The source content of the diagram (plain text).
    pub source: String,

    /// The target output format (e.g., "svg", "png").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// The diagram engine/provider (e.g., "mermaid", "graphviz").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Optional width for the output image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Optional height for the output image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// DTO for a successful diagram rendering response.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderResponseDto {
    /// The rendered output (e.g., SVG text or base64-encoded image).
    pub result: String,

    /// The content type of the result (e.g., "image/svg+xml").
    pub content_type: String,

    /// Rendering duration in milliseconds.
    pub duration_ms: u64,
}
