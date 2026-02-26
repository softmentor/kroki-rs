use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use crate::utils::image_converter;
use anyhow::{Context, Result};
use num_cpus;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use walkdir::WalkDir;

/// Resolves WebP format to the appropriate base format for generation.
fn resolve_base_format<'a>(format: &'a str, type_: &str) -> (&'a str, bool) {
    let is_webp = format.eq_ignore_ascii_case("webp");
    if is_webp {
        if type_.eq_ignore_ascii_case("ditaa") {
            ("png", true)
        } else {
            ("svg", true)
        }
    } else {
        (format, false)
    }
}

/// Core generation pipeline shared by convert and batch.
/// Returns the final output bytes (including optional WebP conversion).
async fn generate_diagram(
    source: &str,
    type_: &str,
    format: &str,
    config: &Config,
    registry: &DiagramRegistry,
    cache_dir: &Option<PathBuf>,
) -> Result<Vec<u8>> {
    let provider = registry.get(type_).context(format!(
        "Diagram type '{}' not supported or tool not found",
        type_
    ))?;

    // Validate input size
    if source.len() > config.server.max_input_size {
        anyhow::bail!(
            "Input too large ({} bytes). Maximum allowed: {} bytes. Configure via server.max_input_size in kroki.toml.",
            source.len(),
            config.server.max_input_size
        );
    }

    // Caching: compute hash (includes fonts & plugin configuration to avoid stale cache)
    let mut fonts = config.all_fonts();
    fonts.sort();
    fonts.dedup();

    let mut plugin_signatures: Vec<String> = config
        .plugins
        .iter()
        .map(|plugin| {
            let args = plugin.args.join(",");
            let formats = plugin.formats.join(",");
            format!(
                "{}|{}|{}|{}|{}|{}",
                plugin.name,
                plugin.command,
                args,
                formats,
                plugin.stdin,
                plugin.timeout_ms.unwrap_or(0)
            )
        })
        .collect();
    plugin_signatures.sort();

    let mut hasher = Sha256::new();
    hasher.update(type_);
    hasher.update(format);
    hasher.update(source);
    hasher.update(b"fonts:");
    for font in &fonts {
        hasher.update(font.as_bytes());
        hasher.update([0]);
    }
    hasher.update(b"plugins:");
    for signature in &plugin_signatures {
        hasher.update(signature.as_bytes());
        hasher.update([0]);
    }
    let hash = hex::encode(hasher.finalize());

    // Check cache
    if let Some(cache_path) = cache_dir {
        if !cache_path.exists() {
            fs::create_dir_all(cache_path).await.ok();
        }
        let cached_file = cache_path.join(format!("{}.{}", hash, format));
        if cached_file.exists() {
            if let Ok(content) = fs::read(&cached_file).await {
                tracing::info!("Cache hit! Served from {}", cached_file.display());
                return Ok(content);
            }
        }
    }

    // Validate
    provider
        .validate(source)
        .context("Source validation failed")?;

    // Generate with format resolution
    let (base_format, is_webp) = resolve_base_format(format, type_);
    let mut output_bytes = provider
        .generate(source, base_format)
        .await
        .context("Diagram generation failed")?;

    // WebP post-processing
    if is_webp {
        let fonts = config.all_fonts();
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
    if let Some(cache_path) = cache_dir {
        let cached_file = cache_path.join(format!("{}.{}", hash, format));
        if let Err(e) = fs::write(&cached_file, &output_bytes).await {
            tracing::warn!("Failed to write to cache: {}", e);
        } else {
            tracing::info!("Saved to cache: {}", cached_file.display());
        }
    }

    Ok(output_bytes)
}

pub async fn convert(
    type_: String,
    format: String,
    input: PathBuf,
    config: Config,
    cache_dir: Option<PathBuf>,
) -> Result<()> {
    let capabilities = Capabilities::discover(&config);
    let browser_manager = match crate::browser::BrowserManager::start(
        config.browser.pool_size,
        config.browser.context_ttl_requests,
    )
    .await
    {
        Ok(m) => Some(Arc::new(m)),
        Err(e) => {
            tracing::warn!("Browser worker failed to start: {}", e);
            None
        }
    };
    let registry = DiagramRegistry::new(&capabilities, &config, browser_manager);
    let cache_dir = Config::resolve_cache_dir(cache_dir);

    let source = fs::read_to_string(&input)
        .await
        .context(format!("Failed to read input file '{}'", input.display()))?;

    let output_bytes =
        generate_diagram(&source, &type_, &format, &config, &registry, &cache_dir).await?;

    let mut stdout = tokio::io::stdout();
    stdout.write_all(&output_bytes).await?;
    stdout.flush().await?;

    Ok(())
}

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

    let capabilities = Capabilities::discover(&config);
    let browser_manager = match crate::browser::BrowserManager::start(
        config.browser.pool_size,
        config.browser.context_ttl_requests,
    )
    .await
    {
        Ok(m) => Some(Arc::new(m)),
        Err(e) => {
            tracing::warn!("Browser worker failed to start: {}", e);
            None
        }
    };
    let registry = Arc::new(DiagramRegistry::new(
        &capabilities,
        &config,
        browser_manager,
    ));
    let config = Arc::new(config);
    let cache_dir = Arc::new(Config::resolve_cache_dir(cache_dir));
    let format = Arc::new(format);
    let type_override = Arc::new(type_override);
    let out_dir = Arc::new(out_dir);

    let mut tasks = Vec::new();
    let failure_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    let concurrency = std::cmp::max(1, num_cpus::get());
    let semaphore = Arc::new(Semaphore::new(concurrency));

    for file_path in files {
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
                "excalidraw" => Some("excalidraw".to_string()),
                "bpmn" => Some("bpmn".to_string()),
                "vega" => Some("vega".to_string()),
                "vl" => Some("vegalite".to_string()),
                _ => {
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
            let registry = registry.clone();
            let cache_dir = cache_dir.clone();
            let format = format.clone();
            let out_dir = out_dir.clone();
            let input_dir = input_dir.clone();
            let failure_count = failure_count.clone();

            let semaphore = semaphore.clone();
            tasks.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await.unwrap();
                let relative_path = file_path.strip_prefix(&input_dir).unwrap_or(&file_path);
                let mut output_path = if let Some(out) = out_dir.as_ref() {
                    out.join(relative_path)
                } else {
                    file_path.clone()
                };
                output_path.set_extension(format.as_str());

                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent).await.ok();
                }

                let source = match fs::read_to_string(&file_path).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Failed to read {}: {}", file_path.display(), e);
                        failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        return;
                    }
                };

                match generate_diagram(&source, &t, &format, &config, &registry, &cache_dir).await {
                    Ok(bytes) => {
                        if let Err(e) = fs::write(&output_path, &bytes).await {
                            tracing::error!("Failed to write {}: {}", output_path.display(), e);
                            failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        } else {
                            tracing::info!(
                                "Converted: {} -> {}",
                                file_path.display(),
                                output_path.display()
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to convert {}: {}", file_path.display(), e);
                        failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }));
        }
    }

    for task in tasks {
        task.await?;
    }

    let failures = failure_count.load(std::sync::atomic::Ordering::Relaxed);
    if failures > 0 {
        anyhow::bail!("{} file(s) failed to convert", failures);
    }

    Ok(())
}
