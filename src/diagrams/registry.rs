use crate::capabilities::Capabilities;
use crate::diagrams::{
    providers::{
        bpmn::BpmnProvider,
        cmd::CommandProvider,
        d2::D2Provider,
        ditaa::DitaaProvider,
        excalidraw::ExcalidrawProvider,
        mermaid::MermaidProvider,
        plantuml::PlantUmlProvider,
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
    pub fn new(capabilities: &Capabilities, config: &crate::config::Config) -> Self {
        let mut providers: HashMap<String, Arc<dyn DiagramProvider + Send + Sync>> = HashMap::new();

        if let Some(path) = &capabilities.graphviz {
            let provider = Arc::new(CommandProvider::new(
                path.clone(),
                config.graphviz.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("graphviz".to_string(), provider.clone());
            providers.insert("dot".to_string(), provider);
        }

        if let Some(path) = &capabilities.mermaid {
            let provider = Arc::new(MermaidProvider::new(
                path.clone(),
                config.mermaid.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("mermaid".to_string(), provider);
        }

        if let Some(path) = &capabilities.plantuml {
            let provider = Arc::new(PlantUmlProvider::new(
                path.clone(),
                config.plantuml.timeout_ms,
            )) as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("plantuml".to_string(), provider.clone());
            providers.insert("c4plantuml".to_string(), provider);
        }

        if let Some(path) = &capabilities.vega {
            let provider = Arc::new(VegaProvider::new(path.clone(), config.vega.timeout_ms))
                as Arc<dyn DiagramProvider + Send + Sync>;
            providers.insert("vega".to_string(), provider);
        }

        if let Some(vl_path) = &capabilities.vegalite {
            // We need vg2svg for vegalite too.
            // In a robust implementation we should ensure capabilities.vega is set,
            // or deduce it from vegalite path if possible.
            // For now, requiring both to be present for vegalite.
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

        if let Some(path) = &capabilities.bpmn {
            let provider = Arc::new(BpmnProvider::new(path.clone(), config.bpmn.timeout_ms))
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

        // Add other providers here

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
