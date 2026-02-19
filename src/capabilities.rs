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
    // Add other tools
}

impl Capabilities {
    /// Discovers available tools based on configuration and system PATH.
    pub fn discover(config: &Config) -> Self {
        let caps = Self {
            graphviz: Self::find_tool(&config.graphviz.bin_path, "dot"),
            mermaid: Self::find_tool(&config.mermaid.bin_path, "mmdc"),
            plantuml: Self::find_tool(&config.plantuml.bin_path, "plantuml"),
            vega: Self::find_tool(&config.vega.bin_path, "vg2svg"),
            vegalite: Self::find_tool(&config.vegalite.bin_path, "vl2vg"),
            wavedrom: Self::find_tool(&config.wavedrom.bin_path, "wavedrom-cli"),
            bpmn: Self::find_tool(&config.bpmn.bin_path, "bpmn-to-image"),
            d2: Self::find_tool(&config.d2.bin_path, "d2"),
            ditaa: Self::find_tool(&config.ditaa.bin_path, "ditaa"),
        };
        // Log discovered tools
        tracing::debug!("Capabilities discovery:");
        tracing::debug!(
            "  Graphviz ({:?}): {:?}",
            config.graphviz.bin_path,
            caps.graphviz
        );
        tracing::debug!(
            "  Mermaid ({:?}): {:?}",
            config.mermaid.bin_path,
            caps.mermaid
        );
        tracing::debug!(
            "  PlantUML ({:?}): {:?}",
            config.plantuml.bin_path,
            caps.plantuml
        );
        tracing::debug!("  Vega ({:?}): {:?}", config.vega.bin_path, caps.vega);
        tracing::debug!(
            "  Vega-Lite ({:?}): {:?}",
            config.vegalite.bin_path,
            caps.vegalite
        );
        tracing::debug!(
            "  Wavedrom ({:?}): {:?}",
            config.wavedrom.bin_path,
            caps.wavedrom
        );
        tracing::debug!("  BPMN ({:?}): {:?}", config.bpmn.bin_path, caps.bpmn);
        tracing::debug!("  D2 ({:?}): {:?}", config.d2.bin_path, caps.d2);
        tracing::debug!("  Ditaa ({:?}): {:?}", config.ditaa.bin_path, caps.ditaa);
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
