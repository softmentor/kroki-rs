use crate::diagrams::error::DiagramError;
use crate::interface::dtos::{RenderRequestDto, RenderResponseDto};

/// Internal domain model for a rendering request.
pub struct DiagramRequest {
    pub source: String,
    pub format: String,
    pub provider: String,
}

impl TryFrom<RenderRequestDto> for DiagramRequest {
    type Error = DiagramError;

    fn try_from(dto: RenderRequestDto) -> Result<Self, Self::Error> {
        if dto.source.is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Source content cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            source: dto.source,
            format: dto.format.ok_or_else(|| {
                DiagramError::ValidationFailed("Format must be specified".to_string())
            })?,
            provider: dto.provider.ok_or_else(|| {
                DiagramError::ValidationFailed("Provider must be specified".to_string())
            })?,
        })
    }
}

impl RenderResponseDto {
    pub fn success(result: Vec<u8>, content_type: &str, duration_ms: u64) -> Self {
        Self {
            result: if content_type.starts_with("text/") || content_type.contains("svg") {
                String::from_utf8_lossy(&result).to_string()
            } else {
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &result)
            },
            content_type: content_type.to_string(),
            duration_ms,
        }
    }
}
