use crate::browser::BrowserManager;
use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use std::sync::Arc;

pub mod admin;
mod handlers;

/// Shared application state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub registry: Arc<DiagramRegistry>,
    pub browser_manager: Option<Arc<BrowserManager>>,
}

/// Starts the Axum web server on the specified port.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let capabilities = Capabilities::discover(&config);
    let port = config.server.port;
    tracing::info!("Capabilities: {:?}", capabilities);
    tracing::info!("Server running on port {}", port);

    let browser_manager = match BrowserManager::start(
        config.browser.pool_size,
        config.browser.context_ttl_requests,
    )
    .await
    {
        Ok(manager) => Some(Arc::new(manager)),
        Err(e) => {
            tracing::warn!("Browser Manager failed to initialize: {}. Playwright-based features will be disabled.", e);
            None
        }
    };

    let registry = Arc::new(DiagramRegistry::new(
        &capabilities,
        &config,
        browser_manager.clone(),
    ));
    let state = AppState {
        config,
        registry,
        browser_manager,
    };

    let admin_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = admin::run_admin_server(admin_state).await {
            tracing::error!("Admin server failed: {}", e);
        }
    });

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
