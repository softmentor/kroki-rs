use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use crate::utils::image_converter;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn convert(
    type_: String,
    format: String,
    input: PathBuf,
    config: Config,
    cache_dir: Option<PathBuf>,
) -> Result<()> {
    let capabilities = Capabilities::discover(&config);
    let registry = DiagramRegistry::new(&capabilities);

    let provider = registry.get(&type_).context(format!(
        "Diagram type '{}' not supported or tool not found",
        type_
    ))?;

    let source = fs::read_to_string(&input)
        .await
        .context(format!("Failed to read input file '{}'", input.display()))?;

    // Caching Logic
    // Compute hash: type + format + source content
    let mut hasher = Sha256::new();
    hasher.update(&type_);
    hasher.update(&format);
    hasher.update(&source);
    let hash = hex::encode(hasher.finalize());

    // Determine cache directory
    let cache_dir = if let Some(d) = cache_dir {
        Some(d)
    } else {
        // Try env var KROKI_CACHE_DIR
        if let Ok(d) = std::env::var("KROKI_CACHE_DIR") {
            Some(PathBuf::from(d))
        } else {
            // Default system cache
            dirs::cache_dir().map(|d| d.join("kroki-rs"))
        }
    };

    if let Some(cache_path) = &cache_dir {
        if !cache_path.exists() {
            fs::create_dir_all(cache_path).await.ok();
        }

        let cached_file = cache_path.join(format!("{}.{}", hash, format));

        if cached_file.exists() {
            if let Ok(content) = fs::read(&cached_file).await {
                let mut stdout = tokio::io::stdout();
                stdout.write_all(&content).await?;
                tracing::info!("Cache hit! Served from {}", cached_file.display());
                return Ok(());
            }
        }
    }

    // Validate if possible
    provider
        .validate(&source)
        .context("Source validation failed")?;

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

    let mut output_bytes = provider
        .generate(&source, base_format)
        .context("Diagram generation failed")?;

    if is_webp {
        let mut fonts = Vec::new();
        fonts.extend_from_slice(&config.mermaid.fonts);
        fonts.extend_from_slice(&config.graphviz.fonts);
        fonts.extend_from_slice(&config.plantuml.fonts);
        fonts.extend_from_slice(&config.excalidraw.fonts);

        output_bytes = if base_format == "png" {
            image_converter::png_to_webp(&output_bytes, image_converter::WebpQuality::Lossless)
                .await
                .context("Failed to convert PNG to WebP")?
        } else {
            image_converter::svg_to_webp(
                &output_bytes,
                image_converter::WebpQuality::Lossless,
                &fonts,
                cache_dir.as_deref(),
            )
            .await
            .context("Failed to convert SVG to WebP")?
        };
    }

    // Write to cache
    if let Some(cache_path) = &cache_dir {
        let cached_file = cache_path.join(format!("{}.{}", hash, format));
        if let Err(e) = fs::write(&cached_file, &output_bytes).await {
            tracing::warn!("Failed to write to cache: {}", e);
        } else {
            tracing::info!("Saved to cache: {}", cached_file.display());
        }
    }

    // For now write to stdout, or maybe derive output filename
    // Just writing to stdout for this simple CLI
    let mut stdout = tokio::io::stdout();
    stdout.write_all(&output_bytes).await?;

    Ok(())
}

use std::sync::Arc;
use walkdir::WalkDir;

pub async fn batch(
    format: String,
    input_dir: PathBuf,
    type_override: Option<String>,
    out_dir: Option<PathBuf>,
    config: Config,
    cache_dir: Option<PathBuf>,
) -> Result<()> {
    if !input_dir.is_dir() {
        return Err(anyhow::anyhow!("Input must be a directory"));
    }

    let files: Vec<PathBuf> = WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_owned())
        .collect();

    tracing::info!("Found {} files in {}", files.len(), input_dir.display());

    let config = Arc::new(config);
    let cache_dir = Arc::new(cache_dir);
    let format = Arc::new(format);
    let type_override = Arc::new(type_override);
    let out_dir = Arc::new(out_dir);

    let mut tasks = Vec::new();

    for file_path in files {
        // Simple heuristic for file extension
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let type_ = if let Some(t) = type_override.as_ref() {
            Some(t.clone())
        } else {
            match extension.as_str() {
                "d2" => Some("d2".to_string()),
                "dot" | "gv" => Some("graphviz".to_string()),
                "mmd" | "mermaid" => Some("mermaid".to_string()),
                "puml" | "plantuml" => Some("plantuml".to_string()),
                "excalidraw" | "json" => {
                    // .json is ambiguous, but if it has "type": "excalidraw" inside...
                    // For now assuming excalidraw extension or user override
                    if extension == "excalidraw" {
                        Some("excalidraw".to_string())
                    } else {
                        None
                    }
                }
                "bpmn" => Some("bpmn".to_string()),
                "vega" => Some("vega".to_string()),
                "vl" => Some("vegalite".to_string()), // .vl.json handled below?
                _ => {
                    // Check for .vl.json
                    if file_path.to_string_lossy().ends_with(".vl.json") {
                        Some("vegalite".to_string())
                    } else {
                        None
                    }
                }
            }
        };

        if let Some(t) = type_ {
            let config = config.clone();
            let cache_dir = cache_dir.clone();
            let format = format.clone();
            let out_dir = out_dir.clone();
            let input_dir = input_dir.clone();

            tasks.push(tokio::spawn(async move {
                // Determine output path
                let relative_path = file_path.strip_prefix(&input_dir).unwrap_or(&file_path);
                let mut output_path = if let Some(out) = out_dir.as_ref() {
                    out.join(relative_path)
                } else {
                    file_path.clone()
                };

                output_path.set_extension(format.as_str());

                // Ensure parent dir exists
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent).await.ok();
                }

                // Call convert_file (refactored from convert)
                // Since convert writes to stdout, we should refactor convert to write to file or return bytes.
                // Refactoring convert to return bytes would be best.
                // BUT `convert` currently does IO.
                // Let's copy-paste logic for now or call `convert_to_file`.

                // Let's duplicate logic for speed, but ideally refactor.
                // Or better: Refactor `convert` to `convert_internal` returning bytes, and `convert` CLI just writes to stdout.

                // Actually, let's just make a private helper `convert_one`
                match convert_to_file(
                    t,
                    format.to_string(),
                    file_path.clone(),
                    output_path.clone(),
                    (*config).clone(),
                    (*cache_dir).clone(),
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!(
                            "Converted: {} -> {}",
                            file_path.display(),
                            output_path.display()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to convert {}: {}", file_path.display(), e);
                    }
                }
            }));
        }
    }

    for task in tasks {
        task.await?;
    }

    Ok(())
}

async fn convert_to_file(
    type_: String,
    format: String,
    input: PathBuf,
    output: PathBuf,
    config: Config,
    cache_dir: Option<PathBuf>,
) -> Result<()> {
    let capabilities = Capabilities::discover(&config);
    let registry = DiagramRegistry::new(&capabilities);

    let provider = registry.get(&type_).context(format!(
        "Diagram type '{}' not supported or tool not found",
        type_
    ))?;

    let source = fs::read_to_string(&input)
        .await
        .context(format!("Failed to read input file '{}'", input.display()))?;

    // Caching Logic
    let mut hasher = Sha256::new();
    hasher.update(&type_);
    hasher.update(&format);
    hasher.update(&source);
    let hash = hex::encode(hasher.finalize());

    let cache_dir = if let Some(d) = cache_dir {
        Some(d)
    } else if let Ok(d) = std::env::var("KROKI_CACHE_DIR") {
        Some(PathBuf::from(d))
    } else {
        dirs::cache_dir().map(|d| d.join("kroki-rs"))
    };

    if let Some(cache_path) = &cache_dir {
        if !cache_path.exists() {
            fs::create_dir_all(cache_path).await.ok();
        }
        let cached_file = cache_path.join(format!("{}.{}", hash, format));
        if cached_file.exists() {
            if let Ok(content) = fs::read(&cached_file).await {
                fs::write(&output, content).await?;
                return Ok(());
            }
        }
    }

    provider
        .validate(&source)
        .context("Source validation failed")?;

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

    let mut output_bytes = provider
        .generate(&source, base_format)
        .context("Diagram generation failed")?;

    if is_webp {
        let mut fonts = Vec::new();
        fonts.extend_from_slice(&config.mermaid.fonts);
        fonts.extend_from_slice(&config.graphviz.fonts);
        fonts.extend_from_slice(&config.plantuml.fonts);
        fonts.extend_from_slice(&config.excalidraw.fonts);

        output_bytes = if base_format == "png" {
            image_converter::png_to_webp(&output_bytes, image_converter::WebpQuality::Lossless)
                .await
                .context("Failed to convert PNG to WebP")?
        } else {
            image_converter::svg_to_webp(
                &output_bytes,
                image_converter::WebpQuality::Lossless,
                &fonts,
                cache_dir.as_deref(),
            )
            .await
            .context("Failed to convert SVG to WebP")?
        };
    }

    if let Some(cache_path) = &cache_dir {
        let cached_file = cache_path.join(format!("{}.{}", hash, format));
        fs::write(&cached_file, &output_bytes).await.ok();
    }

    fs::write(&output, &output_bytes).await?;
    Ok(())
}
