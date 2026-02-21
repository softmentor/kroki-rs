use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Main configuration for Kroki-rs.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
    #[serde(default)]
    pub artifacts: ArtifactsConfig,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    // Tool-specific configurations (TD-03)
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PluginConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_true")]
    pub stdin: bool,
    #[serde(default)]
    pub formats: Vec<String>,
    pub timeout_ms: Option<u64>,
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
    /// Authentication configuration (disabled by default for dev mode).
    #[serde(default)]
    pub auth: AuthConfig,
    /// Rate limiting configuration (disabled by default for dev mode).
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Circuit breaker configuration (disabled by default).
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
    /// Metrics configuration (enabled by default).
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Telemetry/OTel configuration (disabled by default).
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            admin_port: 8081,
            timeout_ms: 5000,
            max_input_size: 1_048_576,   // 1MB
            max_output_size: 52_428_800, // 50MB
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            metrics: MetricsConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ToolConfig {
    pub bin_path: Option<String>,
    pub timeout_ms: Option<u64>,
    pub config_path: Option<String>,
    #[serde(default)]
    pub fonts: Vec<String>,
}

impl ToolConfig {
    pub fn apply_env_overrides(&mut self, prefix: &str) {
        let prefix_upper = prefix.to_uppercase();
        if let Ok(v) = env::var(format!("KROKI_{}_BIN", prefix_upper)) {
            self.bin_path = Some(v);
        }
        if let Ok(v) = env::var(format!("KROKI_{}_TIMEOUT", prefix_upper)) {
            if let Ok(t) = v.parse() {
                self.timeout_ms = Some(t);
            }
        }
        if let Ok(v) = env::var(format!("KROKI_{}_CONFIG", prefix_upper)) {
            self.config_path = Some(v);
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiKeyEntry {
    pub key: String,
    pub label: String,
    /// Optional per-key rate limit (requests per second).
    pub rate_limit: Option<u32>,
}

/// Authentication configuration.
/// When `enabled = false` (default), all auth is bypassed (dev mode).
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    /// List of valid API keys with optional per-key rate limits.
    #[serde(default)]
    pub api_keys: Vec<ApiKeyEntry>,
    /// HTTP header name for API key extraction.
    #[serde(default = "default_auth_header")]
    pub header_name: String,
    /// Bcrpyt hash of the admin password.
    pub admin_password_hash: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys: Vec::new(),
            header_name: "x-api-key".to_string(),
            admin_password_hash: None,
        }
    }
}

/// Rate limiting configuration using token-bucket algorithm.
/// When `enabled = false` (default), no rate limiting is applied.
#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Maximum sustained requests per second.
    #[serde(default = "default_rps")]
    pub requests_per_second: u32,
    /// Maximum burst size above the sustained rate.
    #[serde(default = "default_burst")]
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 10,
            burst_size: 50,
        }
    }
}

/// Circuit breaker configuration for per-provider failure isolation.
/// When `enabled = false` (default), circuit breaker is not applied.
#[derive(Debug, Deserialize, Clone)]
pub struct CircuitBreakerConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Number of consecutive failures before the circuit opens.
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    /// Seconds to wait before transitioning from Open to Half-Open.
    #[serde(default = "default_reset_timeout")]
    pub reset_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_threshold: 5,
            reset_timeout_secs: 30,
        }
    }
}

/// Prometheus metrics configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether to expose a /metrics endpoint on the admin server.
    #[serde(default = "default_false")]
    pub export_endpoint: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_endpoint: true,
        }
    }
}

/// Telemetry (OpenTelemetry) configuration.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    /// OTLP exporter endpoint.
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrowserConfig {
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    /// Number of requests after which a browser context is recreated to prevent memory leaks.
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ArtifactsConfig {
    pub cache_dir: Option<PathBuf>,
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
        if let Ok(password) = env::var("KROKI_ADMIN_PASSWORD") {
            if let Ok(hash) = bcrypt::hash(password, bcrypt::DEFAULT_COST) {
                self.server.auth.admin_password_hash = Some(hash);
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

        // Apply overrides for all tools
        self.graphviz.apply_env_overrides("graphviz");
        self.mermaid.apply_env_overrides("mermaid");
        self.plantuml.apply_env_overrides("plantuml");
        self.vega.apply_env_overrides("vega");
        self.vegalite.apply_env_overrides("vegalite");
        self.wavedrom.apply_env_overrides("wavedrom");
        self.bpmn.apply_env_overrides("bpmn");
        self.d2.apply_env_overrides("d2");
        self.ditaa.apply_env_overrides("ditaa");
        self.excalidraw.apply_env_overrides("excalidraw");
    }

    /// Resolves the cache directory, creating it if it doesn't exist.
    pub fn resolve_cache_dir(custom_path: Option<PathBuf>) -> Option<PathBuf> {
        let path = custom_path.or_else(|| {
            dirs::cache_dir().map(|mut p| {
                p.push("kroki-rs");
                p
            })
        });

        if let Some(ref p) = path {
            let _ = fs::create_dir_all(p);
        }
        path
    }

    /// Aggregates font information from all configured tools.
    pub fn all_fonts(&self) -> Vec<String> {
        let mut fonts = Vec::new();
        fonts.extend(self.graphviz.fonts.clone());
        fonts.extend(self.mermaid.fonts.clone());
        fonts.extend(self.plantuml.fonts.clone());
        fonts.extend(self.vega.fonts.clone());
        fonts.extend(self.vegalite.fonts.clone());
        fonts.extend(self.wavedrom.fonts.clone());
        fonts.extend(self.bpmn.fonts.clone());
        fonts.extend(self.d2.fonts.clone());
        fonts.extend(self.ditaa.fonts.clone());
        fonts.extend(self.excalidraw.fonts.clone());
        fonts
    }
}

// Default helper functions for serde
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
    1_048_576
}
fn default_max_output_size() -> usize {
    52_428_800
}
fn default_auth_header() -> String {
    "x-api-key".to_string()
}
fn default_rps() -> u32 {
    10
}
fn default_burst() -> u32 {
    50
}
fn default_failure_threshold() -> u32 {
    5
}
fn default_reset_timeout() -> u64 {
    30
}
fn default_pool_size() -> usize {
    4
}
fn default_context_ttl() -> usize {
    100
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
