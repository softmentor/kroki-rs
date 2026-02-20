use thiserror::Error;

/// Represents all possible errors that can occur during diagram generation.
#[derive(Debug, Error)]
pub enum DiagramError {
    /// The input diagram source failed validation.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// The required underlying tool or binary was not found on the system.
    #[error("Tool not found: '{0}'. Is it installed and in your PATH?")]
    ToolNotFound(String),

    /// Execution of the tool timed out.
    #[error("'{tool}' timed out after {timeout_ms}ms (input: {bytes} bytes)")]
    ExecutionTimeout {
        tool: String,
        timeout_ms: u64,
        bytes: usize,
    },

    /// The tool process executed but returned a non-zero exit code or failed to yield expected output.
    #[error("Process execution failed: {0}")]
    ProcessFailed(String),

    /// Base64 decoding, Zlib decompression, or UTF-8 conversion failed.
    #[error("Failed to decode requested string: {0}")]
    DecodeFailed(String),

    /// The requested output format is not supported by this provider.
    #[error("Format '{format}' is not supported by provider '{provider}'")]
    UnsupportedFormat { format: String, provider: String },

    /// Catch-all for IO or system-level IO errors during execution.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An unexpected error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// A specialized Result type for diagram generation operations.
pub type DiagramResult<T> = std::result::Result<T, DiagramError>;
