use crate::browser::BrowserManager;
use crate::capabilities::Capabilities;
use crate::config::Config;
use crate::diagrams::registry::DiagramRegistry;
use std::sync::Arc;

pub mod admin;
mod handlers;
pub mod metrics;
pub mod middleware;

use metrics::PrometheusHandle;
use middleware::circuit_breaker::CircuitBreakerManager;
use middleware::rate_limit::RateLimiter;

/// Shared application state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub registry: Arc<DiagramRegistry>,
    pub browser_manager: Option<Arc<BrowserManager>>,
    pub rate_limiter: Option<RateLimiter>,
    pub circuit_breaker: Option<CircuitBreakerManager>,
    pub metrics_handle: Option<PrometheusHandle>,
}

/// Starts the Axum web server on the specified port.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let capabilities = Capabilities::discover(&config);
    let port = config.server.port;
    tracing::info!("Capabilities: {:?}", capabilities);
    tracing::info!(
        "Kroki-rs discovery service available at http://localhost:{}",
        port
    );

    let browser_manager = match BrowserManager::start(
        config.browser.pool_size,
        config.browser.context_ttl_requests,
    )
    .await
    {
        Ok(manager) => Some(Arc::new(manager)),
        Err(e) => {
            tracing::warn!("Native Browser Backend failed to initialize: {}. Browser-based features (Mermaid, BPMN) will be disabled.", e);
            None
        }
    };

    let registry = Arc::new(DiagramRegistry::new(
        &capabilities,
        &config,
        browser_manager.clone(),
    ));

    // Initialize rate limiter if enabled
    let rate_limiter = if config.server.rate_limit.enabled {
        tracing::info!(
            "Rate limiting enabled: {} req/s, burst: {}",
            config.server.rate_limit.requests_per_second,
            config.server.rate_limit.burst_size
        );
        Some(RateLimiter::new(&config.server.rate_limit))
    } else {
        tracing::info!("Rate limiting disabled (dev mode)");
        None
    };

    // Initialize circuit breaker if enabled
    let circuit_breaker = if config.server.circuit_breaker.enabled {
        tracing::info!(
            "Circuit breaker enabled: threshold={}, reset={}s",
            config.server.circuit_breaker.failure_threshold,
            config.server.circuit_breaker.reset_timeout_secs
        );
        Some(CircuitBreakerManager::new(&config.server.circuit_breaker))
    } else {
        tracing::info!("Circuit breaker disabled");
        None
    };

    // Initialize metrics if enabled
    let metrics_handle = if config.server.metrics.enabled {
        tracing::info!("Metrics collection enabled");
        Some(metrics::init_metrics())
    } else {
        tracing::info!("Metrics collection disabled");
        None
    };

    if config.server.auth.enabled {
        tracing::info!(
            "API key authentication enabled ({} key(s) configured)",
            config.server.auth.api_keys.len()
        );
    } else {
        tracing::info!("Authentication disabled (dev mode)");
    }

    let state = AppState {
        config,
        registry,
        browser_manager,
        rate_limiter,
        circuit_breaker,
        metrics_handle,
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
    use axum::{middleware as mw, routing::get};

    axum::Router::new()
        .route("/", get(handlers::root))
        .route("/{type}/{format}/{source}", get(handlers::get_diagram))
        .route(
            "/{type}/{format}",
            axum::routing::post(handlers::post_render),
        )
        .layer(mw::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(mw::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ))
        .with_state(state)
}
