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

    let host = state.config.server.host.clone();
    let app = app
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::server::middleware::auth::admin_auth_middleware,
        ))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Admin dashboard available at http://{}:{}", host, port);
    if metrics_export_enabled {
        tracing::info!(
            "Prometheus metrics available at http://{}:{}/metrics",
            host,
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

    let api_port = state.config.server.port;
    let host = &state.config.server.host;

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kroki-rs | Admin Dashboard</title>
    <style>
        :root {{
            --primary: #6366f1;
            --primary-dark: #4f46e5;
            --bg: #0f172a;
            --card-bg: #1e293b;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --success: #22c55e;
            --warning: #eab308;
        }}
        body {{
            font-family: 'Inter', -apple-system, sans-serif;
            background-color: var(--bg);
            color: var(--text);
            margin: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            min-height: 100vh;
            padding: 2rem;
        }}
        .container {{
            max-width: 900px;
            width: 100%;
        }}
        header {{
            text-align: center;
            margin-bottom: 3rem;
        }}
        h1 {{
            font-size: 2.5rem;
            margin: 0;
            background: linear-gradient(to right, #818cf8, #c084fc);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .version {{
            font-size: 0.875rem;
            color: var(--text-muted);
            margin-top: 0.5rem;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }}
        .card {{
            background: var(--card-bg);
            border-radius: 1rem;
            padding: 1.5rem;
            border: 1px solid #334155;
        }}
        .card h2 {{
            margin-top: 0;
            font-size: 1.25rem;
            border-bottom: 1px solid #334155;
            padding-bottom: 0.75rem;
            margin-bottom: 1rem;
        }}
        .status-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 0.75rem;
        }}
        .status-label {{ color: var(--text-muted); }}
        .status-value {{ font-weight: 600; }}
        .enabled {{ color: var(--success); }}
        .disabled {{ color: var(--text-muted); }}
        
        ul {{
            columns: 2;
            list-style-type: none;
            padding: 0;
            margin: 0;
        }}
        li {{
            padding: 0.25rem 0;
            font-size: 0.875rem;
        }}
        
        .nav-link {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--primary);
            text-decoration: none;
            font-weight: 500;
            margin-bottom: 2rem;
        }}
        .nav-link:hover {{ text-decoration: underline; }}

        .footer {{
            margin-top: auto;
            padding-top: 3rem;
            color: var(--text-muted);
            font-size: 0.875rem;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <a href="http://{}:{}" class="nav-link">🏠 Back to Discovery</a>
            <h1>Admin Dashboard</h1>
            <div class="version">Kroki-rs v{}</div>
        </header>
        
        <div class="grid">
            <div class="card">
                <h2>System Status</h2>
                <div class="status-row">
                    <span class="status-label">Service</span>
                    <span class="status-value enabled">Online</span>
                </div>
                <div class="status-row">
                    <span class="status-label">Authentication</span>
                    <span class="status-value {}">{}</span>
                </div>
                <div class="status-row">
                    <span class="status-label">Rate Limiting</span>
                    <span class="status-value {}">{}</span>
                </div>
                <div class="status-row">
                    <span class="status-label">Circuit Breaker</span>
                    <span class="status-value {}">{}</span>
                </div>
            </div>

            <div class="card">
                <h2>Diagram Providers</h2>
                <div style="font-size: 0.875rem; color: var(--text-muted); margin-bottom: 1rem;">
                    {} available tools registered
                </div>
                <ul>
                    {}
                </ul>
            </div>
        </div>

        <div class="footer">
            Built with Rust & Axum • <a href="https://github.com/softmentor/kroki-rs" style="color: inherit;">GitHub</a>
        </div>
    </div>
</body>
</html>
"#,
        host,
        api_port,
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
