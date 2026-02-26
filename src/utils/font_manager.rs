use anyhow::{Context, Result};
use hex::encode as hex_encode;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;

/// Manages downloading and caching of custom TTF/OTF fonts.
pub struct FontManager {
    cache_dir: PathBuf,
    client: Client,
}

impl FontManager {
    pub fn new(cache_dir: Option<&Path>) -> Result<Self> {
        let dir = if let Some(d) = cache_dir {
            d.join("fonts")
        } else {
            crate::config::Config::resolve_cache_dir(None)
                .map(|d| d.join("fonts"))
                .unwrap_or_else(|| PathBuf::from(".kroki-fonts"))
        };

        std::fs::create_dir_all(&dir)?;

        Ok(Self {
            cache_dir: dir,
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .context("Failed to build font HTTP client")?,
        })
    }

    /// Takes a list of URLs (or local paths) and ensures they are available in the cache directory.
    /// Returns the path to the font cache directory.
    pub async fn prepare_fonts(&self, urls: &[String]) -> Result<PathBuf> {
        if urls.is_empty() {
            return Ok(self.cache_dir.clone());
        }

        let mut futures = Vec::new();

        for url in urls {
            let file_name = Self::safe_file_name(url);
            let local_path = self.cache_dir.join(&file_name);

            if !local_path.exists() {
                let url_clone = url.clone();
                let client = self.client.clone();

                futures.push(tokio::spawn(async move {
                    tracing::info!("Downloading custom font: {}", url_clone);
                    if Self::is_remote(&url_clone) {
                        let resp = client.get(&url_clone).send().await?.error_for_status()?;
                        let content_length = resp.content_length();
                        let bytes = resp.bytes().await?;
                        Self::validate_font_size(content_length, bytes.len())?;
                        fs::write(&local_path, &bytes).await?;
                    } else {
                        let path = PathBuf::from(&url_clone);
                        let metadata = fs::metadata(&path).await?;
                        Self::validate_font_size(Some(metadata.len()), metadata.len() as usize)?;
                        let data = fs::read(&path).await?;
                        fs::write(&local_path, &data).await?;
                    }
                    Ok::<_, anyhow::Error>(())
                }));
            }
        }

        for f in futures {
            f.await??;
        }

        Ok(self.cache_dir.clone())
    }

    fn is_remote(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    fn safe_file_name(source: &str) -> String {
        let hash = Sha256::digest(source.as_bytes());
        let mut name = hex_encode(hash);

        let segment = source
            .rsplit('/')
            .next()
            .and_then(|segment| segment.split('?').next())
            .and_then(|segment| segment.split('#').next())
            .filter(|segment| !segment.is_empty());

        if let Some(segment) = segment {
            if let Some(ext) = Path::new(segment).extension().and_then(|e| e.to_str()) {
                name.push('.');
                name.push_str(&ext.to_lowercase());
                return name;
            }
        }

        name.push_str(".ttf");
        name
    }

    fn validate_font_size(content_length: Option<u64>, actual: usize) -> Result<()> {
        const MAX_FONT_BYTES: usize = 5 * 1024 * 1024; // 5 MiB

        if actual > MAX_FONT_BYTES {
            anyhow::bail!(
                "Font payload too large ({} bytes). Maximum allowed: {} bytes",
                actual,
                MAX_FONT_BYTES
            );
        }
        if let Some(expected) = content_length {
            if expected as usize > MAX_FONT_BYTES {
                anyhow::bail!(
                    "Font payload too large based on Content-Length ({} bytes)",
                    expected
                );
            }
        }

        Ok(())
    }
}
