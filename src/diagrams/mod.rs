pub mod providers {
    pub mod bpmn;
    pub mod cmd;
    pub mod d2;
    pub mod ditaa;
    pub mod mermaid;
    pub mod plantuml;
    pub mod vega;
    pub mod wavedrom;
}
pub mod registry;

use anyhow::Result;

/// A trait for diagram generation providers.
///
/// Each provider implementation is responsible for a specific diagram type
/// (e.g., Mermaid, PlantUML, Graphviz).
pub trait DiagramProvider {
    /// Validates the diagram source text.
    ///
    /// Returns `Ok(())` if the source is valid, or an error otherwise.
    fn validate(&self, source: &str) -> Result<()>;

    /// Generates a diagram image from the source text.
    ///
    /// # Arguments
    /// * `source` - The diagram description text.
    /// * `format` - The desired output format (e.g., "svg", "png").
    ///
    /// Returns a `Vec<u8>` containing the image data.
    fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>>;
}
