use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn convert(type_: String, format: String, input: PathBuf, config: Config) -> Result<()> {
    let capabilities = Capabilities::discover(&config);
    let registry = DiagramRegistry::new(&capabilities);

    let provider = registry.get(&type_).context(format!(
        "Diagram type '{}' not supported or tool not found",
        type_
    ))?;

    let source = fs::read_to_string(&input)
        .await
        .context(format!("Failed to read input file '{}'", input.display()))?;

    // Validate if possible
    provider
        .validate(&source)
        .context("Source validation failed")?;

    let output_bytes = provider
        .generate(&source, &format)
        .context("Diagram generation failed")?;

    // For now write to stdout, or maybe derive output filename
    // Just writing to stdout for this simple CLI
    let mut stdout = tokio::io::stdout();
    stdout.write_all(&output_bytes).await?;

    Ok(())
}
