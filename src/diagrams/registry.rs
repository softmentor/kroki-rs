use crate::browser::BrowserManager;
use crate::capabilities::Capabilities;
use crate::diagrams::{
    providers::{
        bpmn::BpmnProvider,
        cmd::CommandProvider,
        d2::D2Provider,
        ditaa::DitaaProvider,
        excalidraw::ExcalidrawProvider,
        mermaid::MermaidProvider,
        vega::{VegaLiteProvider, VegaProvider},
        wavedrom::WavedromProvider,
    },
    DiagramProvider,
};
use std::collections::HashMap;
use std::sync::Arc;

pub struct DiagramRegistry {
    providers: HashMap<String, Arc<dyn DiagramProvider + Send + Sync>>,
}

impl DiagramRegistry {
    pub fn new(
        capabilities: &Capabilities,
        config: &crate::config::Config,
        browser_manager: Option<Arc<BrowserManager>>,
    ) -> Self {
        let mut providers: HashMap<String, Arc<dyn DiagramProvider + Send + Sync>> = HashMap::new();

        if let Some(path) = &capabilities.graphviz {
            let provider = Arc::new(CommandProvider::new(
                path.clone(),
                config.graphviz.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("graphviz".to_string(), provider.clone());
            providers.insert("dot".to_string(), provider);
        }

        if let (Some(_), Some(browser)) = (&capabilities.mermaid, &browser_manager) {
            let provider = Arc::new(MermaidProvider::new(
                browser.clone(),
                config.mermaid.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("mermaid".to_string(), provider);
        }

        if let Some(path) = &capabilities.vega {
            let provider = Arc::new(VegaProvider::new(path.clone(), config.vega.timeout_ms))
                as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("vega".to_string(), provider);
        }

        if let Some(vl_path) = &capabilities.vegalite {
            if let Some(vg_path) = &capabilities.vega {
                let provider = Arc::new(VegaLiteProvider::new(
                    vl_path.clone(),
                    vg_path.clone(),
                    config.vegalite.timeout_ms,
                )) as Arc<dyn DiagramProvider + Send + Sync>;
                providers.insert("vegalite".to_string(), provider);
            }
        }

        if let Some(path) = &capabilities.wavedrom {
            let provider = Arc::new(WavedromProvider::new(
                path.clone(),
                config.wavedrom.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("wavedrom".to_string(), provider);
        }

        if let (Some(_), Some(browser)) = (&capabilities.bpmn, &browser_manager) {
            let provider = Arc::new(BpmnProvider::new(browser.clone(), config.bpmn.timeout_ms))
                as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("bpmn".to_string(), provider);
        }

        if let Some(path) = &capabilities.d2 {
            let provider = Arc::new(D2Provider::new(path.clone(), config.d2.timeout_ms))
                as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("d2".to_string(), provider);
        }

        if let Some(path) = &capabilities.ditaa {
            let provider = Arc::new(DitaaProvider::new(path.clone(), config.ditaa.timeout_ms))
                as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("ditaa".to_string(), provider);
        }

        if let Some(path) = &capabilities.excalidraw {
            let provider = Arc::new(ExcalidrawProvider::new(
                path.clone(),
                config.excalidraw.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("excalidraw".to_string(), provider);
        }

        // 2. Register custom plugins (TD-05 / ADR 0007)
        for plugin_cfg in &config.plugins {
            if providers.contains_key(&plugin_cfg.name) {
                tracing::error!(
                    "Plugin collision: '{}' already exists as a built-in provider. Skipping.",
                    plugin_cfg.name
                );
                continue;
            }

            tracing::info!("Registering custom plugin: {}", plugin_cfg.name);
            let provider = Arc::new(crate::diagrams::providers::plugin::PluginProvider::new(
                plugin_cfg,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert(plugin_cfg.name.clone(), provider);
        }

        Self { providers }
    }

    /// Returns the provider for a given diagram type, if available.
    pub fn get(&self, name: &str) -> Option<Arc<dyn DiagramProvider + Send + Sync>> {
        self.providers.get(name).cloned()
    }

    /// Returns the list of all registered diagram type names.
    pub fn known_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.providers.keys().cloned().collect();
        types.sort();
        types
    }
}
