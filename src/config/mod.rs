use serde::Deserialize;
use std::path::PathBuf;
use std::{env, fs};

/// The global configuration for Kroki-rs.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
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
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            timeout_ms: 5000,
        }
    }
}

fn default_port() -> u16 {
    8000
}
fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ToolConfig {
    pub bin_path: Option<String>,
    pub config_path: Option<String>,
}

impl Config {
    /// Loads the configuration from a path, environment variable, or default file.
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = if let Some(p) = path {
            p
        } else if let Ok(p) = env::var("KROKI_CONFIG") {
            PathBuf::from(p)
        } else if fs::metadata("kroki.toml").is_ok() {
            PathBuf::from("kroki.toml")
        } else {
            return Ok(Config::default());
        };

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
