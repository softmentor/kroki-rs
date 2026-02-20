use serde::Deserialize;
use std::path::PathBuf;
use std::{env, fs};

/// The global configuration for Kroki-rs.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
    #[serde(default)]
    pub graphviz: ToolConfig,
    #[serde(default)]
    pub mermaid: ToolConfig,
    #[serde(default)]
    pub plantuml: ToolConfig,
    #[serde(default)]
    pub vega: ToolConfig,
    #[serde(default)]
    pub vegalite: ToolConfig,
    #[serde(default)]
    pub wavedrom: ToolConfig,
    #[serde(default)]
    pub bpmn: ToolConfig,
    #[serde(default)]
    pub d2: ToolConfig,
    #[serde(default)]
    pub ditaa: ToolConfig,
    #[serde(default)]
    pub excalidraw: ToolConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            browser: BrowserConfig::default(),
            graphviz: ToolConfig {
                bin_path: Some("dot".into()),
                ..Default::default()
            },
            mermaid: ToolConfig {
                bin_path: Some("mmdc".into()),
                ..Default::default()
            },
            plantuml: ToolConfig {
                bin_path: Some("plantuml".into()),
                ..Default::default()
            },
            vega: ToolConfig {
                bin_path: Some("vg2svg".into()),
                ..Default::default()
            },
            vegalite: ToolConfig {
                bin_path: Some("vl2vg".into()),
                ..Default::default()
            },
            wavedrom: ToolConfig {
                bin_path: Some("wavedrom-cli".into()),
                ..Default::default()
            },
            bpmn: ToolConfig {
                bin_path: Some("bpmn-to-image".into()),
                ..Default::default()
            },
            d2: ToolConfig {
                bin_path: Some("d2".into()),
                ..Default::default()
            },
            ditaa: ToolConfig {
                bin_path: Some("ditaa".into()),
                ..Default::default()
            },
            excalidraw: ToolConfig {
                bin_path: Some("excalidraw-to-svg".into()),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Maximum allowed input size in bytes (default: 1MB).
    #[serde(default = "default_max_input_size")]
    pub max_input_size: usize,
    /// Maximum allowed output size in bytes (default: 50MB).
    #[serde(default = "default_max_output_size")]
    pub max_output_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            admin_port: 8081,
            timeout_ms: 5000,
            max_input_size: 1_048_576,   // 1MB
            max_output_size: 52_428_800, // 50MB
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrowserConfig {
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    #[serde(default = "default_context_ttl")]
    pub context_ttl_requests: usize,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            pool_size: 4,
            context_ttl_requests: 100,
        }
    }
}

fn default_pool_size() -> usize {
    4
}
fn default_context_ttl() -> usize {
    100
}

fn default_port() -> u16 {
    8000
}
fn default_admin_port() -> u16 {
    8081
}
fn default_timeout() -> u64 {
    5000
}
fn default_max_input_size() -> usize {
    1_048_576 // 1MB
}
fn default_max_output_size() -> usize {
    52_428_800 // 50MB
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ToolConfig {
    pub bin_path: Option<String>,
    pub config_path: Option<String>,
    #[serde(default)]
    pub fonts: Vec<String>,
    pub timeout_ms: Option<u64>,
}

impl ToolConfig {
    pub fn apply_env_overrides(&mut self, prefix: &str) {
        if let Ok(v) = env::var(format!("KROKI_{}_BIN", prefix)) {
            self.bin_path = Some(v);
        }
        if let Ok(v) = env::var(format!("KROKI_{}_TIMEOUT", prefix)) {
            if let Ok(t) = v.parse() {
                self.timeout_ms = Some(t);
            }
        }
        if let Ok(v) = env::var(format!("KROKI_{}_CONFIG", prefix)) {
            self.config_path = Some(v);
        }
    }
}

/// Supported output formats.
pub const SUPPORTED_FORMATS: &[&str] = &["svg", "png", "pdf", "webp", "txt"];

impl Config {
    /// Loads the configuration from a path, environment variable, or default file.
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = if let Some(p) = path {
            Some(p)
        } else if let Ok(p) = env::var("KROKI_CONFIG") {
            Some(PathBuf::from(p))
        } else if fs::metadata("kroki.toml").is_ok() {
            Some(PathBuf::from("kroki.toml"))
        } else {
            None
        };

        let mut config = if let Some(p) = path {
            let content = fs::read_to_string(p)?; // TODO handle toml error nicely?
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        config.apply_env_overrides();

        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(v) = env::var("KROKI_PORT") {
            if let Ok(p) = v.parse() {
                self.server.port = p;
            }
        }
        if let Ok(v) = env::var("KROKI_ADMIN_PORT") {
            if let Ok(p) = v.parse() {
                self.server.admin_port = p;
            }
        }
        if let Ok(v) = env::var("KROKI_TIMEOUT") {
            if let Ok(t) = v.parse() {
                self.server.timeout_ms = t;
            }
        }
        if let Ok(v) = env::var("KROKI_MAX_INPUT_SIZE") {
            if let Ok(s) = v.parse() {
                self.server.max_input_size = s;
            }
        }
        if let Ok(v) = env::var("KROKI_MAX_OUTPUT_SIZE") {
            if let Ok(s) = v.parse() {
                self.server.max_output_size = s;
            }
        }

        if let Ok(v) = env::var("KROKI_BROWSER_POOL_SIZE") {
            if let Ok(s) = v.parse() {
                self.browser.pool_size = s;
            }
        }

        if let Ok(v) = env::var("KROKI_BROWSER_CONTEXT_TTL") {
            if let Ok(s) = v.parse() {
                self.browser.context_ttl_requests = s;
            }
        }

        self.graphviz.apply_env_overrides("GRAPHVIZ");
        self.mermaid.apply_env_overrides("MERMAID");
        self.plantuml.apply_env_overrides("PLANTUML");
        self.vega.apply_env_overrides("VEGA");
        self.vegalite.apply_env_overrides("VEGALITE");
        self.wavedrom.apply_env_overrides("WAVEDROM");
        self.bpmn.apply_env_overrides("BPMN");
        self.d2.apply_env_overrides("D2");
        self.ditaa.apply_env_overrides("DITAA");
        self.excalidraw.apply_env_overrides("EXCALIDRAW");
    }

    /// Collects all font URLs from every tool configuration.
    pub fn all_fonts(&self) -> Vec<String> {
        let mut fonts = Vec::new();
        fonts.extend_from_slice(&self.mermaid.fonts);
        fonts.extend_from_slice(&self.graphviz.fonts);
        fonts.extend_from_slice(&self.plantuml.fonts);
        fonts.extend_from_slice(&self.excalidraw.fonts);
        fonts
    }

    /// Resolves the cache directory from CLI override, env var, or system default.
    pub fn resolve_cache_dir(cli_override: Option<PathBuf>) -> Option<PathBuf> {
        if let Some(d) = cli_override {
            Some(d)
        } else if let Ok(d) = env::var("KROKI_CACHE_DIR") {
            Some(PathBuf::from(d))
        } else {
            dirs::cache_dir().map(|d| d.join("kroki-rs"))
        }
    }
}
