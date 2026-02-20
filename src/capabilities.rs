use crate::config::Config;
use std::path::PathBuf;
use which::which;

/// Holds the paths to discovered diagram generation tools.
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub graphviz: Option<PathBuf>,
    pub mermaid: Option<PathBuf>,
    pub plantuml: Option<PathBuf>,
    pub vega: Option<PathBuf>,     // vg2svg
    pub vegalite: Option<PathBuf>, // vl2vg
    pub wavedrom: Option<PathBuf>,
    pub bpmn: Option<PathBuf>,
    pub d2: Option<PathBuf>,
    pub ditaa: Option<PathBuf>,
    pub excalidraw: Option<PathBuf>,
    // Add other tools
}

impl Capabilities {
    /// Discovers available tools based on configuration and system PATH.
    pub fn discover(config: &Config) -> Self {
        let caps = Self {
            graphviz: Self::find_tool(&config.graphviz.bin_path, "dot"),
            // Playwright integrated tools rely purely on the Node.js global binaries
            mermaid: Self::find_tool(&None, "node"),
            plantuml: Self::find_tool(&config.plantuml.bin_path, "plantuml"),
            vega: Self::find_tool(&config.vega.bin_path, "vg2svg"),
            vegalite: Self::find_tool(&config.vegalite.bin_path, "vl2vg"),
            wavedrom: Self::find_tool(&config.wavedrom.bin_path, "wavedrom-cli"),
            bpmn: Self::find_tool(&None, "node"),
            d2: Self::find_tool(&config.d2.bin_path, "d2"),
            ditaa: Self::find_tool(&config.ditaa.bin_path, "ditaa"),
            excalidraw: Self::find_tool(&config.excalidraw.bin_path, "excalidraw-to-svg"),
        };
        // Log summary of discovered capabilities
        let found: Vec<&str> = [
            caps.graphviz.as_ref().map(|_| "graphviz"),
            caps.mermaid.as_ref().map(|_| "mermaid"),
            caps.plantuml.as_ref().map(|_| "plantuml"),
            caps.vega.as_ref().map(|_| "vega"),
            caps.vegalite.as_ref().map(|_| "vegalite"),
            caps.wavedrom.as_ref().map(|_| "wavedrom"),
            caps.bpmn.as_ref().map(|_| "bpmn"),
            caps.d2.as_ref().map(|_| "d2"),
            caps.ditaa.as_ref().map(|_| "ditaa"),
            caps.excalidraw.as_ref().map(|_| "excalidraw"),
        ]
        .into_iter()
        .flatten()
        .collect();
        tracing::info!("Discovered {} tools: [{}]", found.len(), found.join(", "));
        caps
    }

    /// Finds a tool by checking the configured path, local node_modules, and system PATH.
    fn find_tool(configured_path: &Option<String>, default_name: &str) -> Option<PathBuf> {
        if let Some(path_str) = configured_path {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            } else if let Ok(p) = which(path_str) {
                return Some(p);
            }
        }

        let local_node_bin = PathBuf::from("node_modules/.bin").join(default_name);
        if local_node_bin.exists() {
            return Some(local_node_bin);
        }

        which(default_name).ok()
    }
}
