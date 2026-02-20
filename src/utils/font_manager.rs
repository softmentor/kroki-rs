use anyhow::Result;
use reqwest::Client;
use std::path::{Path, PathBuf};
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
            client: Client::new(),
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
            // Very simple file naming based on the URL's trailing segment or a hash
            let file_name = url.split('/').next_back().unwrap_or("custom_font.ttf");
            let local_path = self.cache_dir.join(file_name);

            if !local_path.exists() {
                let url_clone = url.clone();
                let client = self.client.clone();

                futures.push(tokio::spawn(async move {
                    tracing::info!("Downloading custom font: {}", url_clone);
                    let resp = client.get(&url_clone).send().await?.error_for_status()?;
                    let bytes = resp.bytes().await?;
                    fs::write(&local_path, &bytes).await?;
                    Ok::<_, anyhow::Error>(())
                }));
            }
        }

        for f in futures {
            f.await??;
        }

        Ok(self.cache_dir.clone())
    }
}
