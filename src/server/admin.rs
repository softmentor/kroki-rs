use crate::server::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use tokio::net::TcpListener;

/// Starts the admin server alongside the main application.
pub async fn run_admin_server(state: AppState) -> anyhow::Result<()> {
    let port = state.config.server.admin_port;
    let metrics_export_enabled =
        state.config.server.metrics.enabled && state.config.server.metrics.export_endpoint;

    let mut app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(dashboard));

    // Add metrics endpoint if enabled and export is configured
    if metrics_export_enabled {
        app = app.route("/metrics", get(metrics_handler));
    }

    let app = app
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::server::middleware::auth::admin_auth_middleware,
        ))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Admin dashboard available at http://localhost:{}", port);
    if metrics_export_enabled {
        tracing::info!(
            "Prometheus metrics available at http://localhost:{}/metrics",
            port
        );
    }
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut health_data = json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    });

    if let Some(browser) = &state.browser_manager {
        if let Ok(pool_health) = browser.get_pool_health().await {
            health_data["browser_pool"] = pool_health;
        } else {
            health_data["browser_pool"] = json!({"status": "unhealthy"});
        }
    }

    Json(health_data)
}

async fn dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let capabilities = state.registry.known_types();
    let auth_status = if state.config.server.auth.enabled {
        "Enabled"
    } else {
        "Disabled (Dev Mode)"
    };
    let rate_limit_status = if state.config.server.rate_limit.enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    let cb_status = if state.config.server.circuit_breaker.enabled {
        "Enabled"
    } else {
        "Disabled"
    };

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Kroki-rs Dashboard</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; padding: 2rem; background: #f9fafb; color: #111827; }}
        h1 {{ border-bottom: 2px solid #e5e7eb; padding-bottom: 0.5rem; }}
        .card {{ background: white; padding: 1.5rem; border-radius: 0.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 1rem; }}
        .status {{ font-weight: bold; }}
        .enabled {{ color: #059669; }}
        .disabled {{ color: #6b7280; }}
        ul {{ columns: 2; list-style-type: none; padding: 0; }}
        li {{ padding: 0.25rem 0; }}
        .feature-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; }}
    </style>
</head>
<body>
    <h1>Kroki-rs v{} Admin Dashboard</h1>
    
    <div class="card">
        <h2>System Status</h2>
        <div class="feature-grid">
            <div>
                <p>Service: <span class="status enabled">Online</span></p>
                <p>Authentication: <span class="status {}">{}</span></p>
            </div>
            <div>
                <p>Rate Limiting: <span class="status {}">{}</span></p>
                <p>Circuit Breaker: <span class="status {}">{}</span></p>
            </div>
        </div>
    </div>

    <div class="card">
        <h2>Supported Diagram Types ({} available)</h2>
        <ul>
            {}
        </ul>
    </div>
</body>
</html>
"#,
        env!("CARGO_PKG_VERSION"),
        if state.config.server.auth.enabled {
            "enabled"
        } else {
            "disabled"
        },
        auth_status,
        if state.config.server.rate_limit.enabled {
            "enabled"
        } else {
            "disabled"
        },
        rate_limit_status,
        if state.config.server.circuit_breaker.enabled {
            "enabled"
        } else {
            "disabled"
        },
        cb_status,
        capabilities.len(),
        capabilities
            .iter()
            .map(|c| format!("<li>✅ {}</li>", c))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Html(html)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(handle) = &state.metrics_handle {
        handle.render()
    } else {
        "Metrics collection is disabled".to_string()
    }
}
