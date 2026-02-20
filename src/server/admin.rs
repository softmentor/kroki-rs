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
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(dashboard))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Admin dashboard available at http://localhost:{}", port);
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
        .success {{ color: #059669; font-weight: bold; }}
        ul {{ columns: 2; list-style-type: none; padding: 0; }}
        li {{ padding: 0.25rem 0; }}
    </style>
</head>
<body>
    <h1>Kroki-rs v{} Admin Dashboard</h1>
    
    <div class="card">
        <h2>System Status</h2>
        <p>Service: <span class="success">Online</span></p>
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
        capabilities.len(),
        capabilities
            .iter()
            .map(|c| format!("<li>✅ {}</li>", c))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Html(html)
}
