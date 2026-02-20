use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use std::sync::Arc;

mod handlers;

/// Shared application state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub registry: Arc<DiagramRegistry>,
}

/// Starts the Axum web server on the specified port.
pub async fn run(port: u16, config: Config) -> anyhow::Result<()> {
    let capabilities = Capabilities::discover(&config);
    tracing::info!("Capabilities: {:?}", capabilities);
    tracing::info!("Server running on port {}", port);

    let registry = Arc::new(DiagramRegistry::new(&capabilities, &config));
    let state = AppState { config, registry };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app(state)).await?;

    Ok(())
}

fn app(state: AppState) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        .route("/", get(handlers::root))
        .route("/{type}/{format}/{source}", get(handlers::get_diagram))
        .with_state(state)
}
