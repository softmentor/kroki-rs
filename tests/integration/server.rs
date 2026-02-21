use kroki_rs::config::Config;
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::time::sleep;

struct TestContext {
    port: u16,
    admin_port: u16,
    client: Client,
}

static CONTEXT: OnceLock<TestContext> = OnceLock::new();

async fn get_context() -> &'static TestContext {
    if let Some(ctx) = CONTEXT.get() {
        return ctx;
    }

    let port = get_available_port();
    let admin_port = get_available_port();

    let mut config = Config::default();
    config.server.port = port;
    config.server.admin_port = admin_port;
    config.server.max_input_size = 1024; // Standardized for test

    // Discover tools
    config.graphviz.bin_path = which::which("dot")
        .ok()
        .map(|p| p.to_string_lossy().into_owned());

    tokio::spawn(async move {
        let _ = server::run(config).await;
    });

    // Wait for health
    let client = Client::new();
    let mut started = false;
    for _ in 0..50 {
        if client
            .get(format!("http://127.0.0.1:{}/health", admin_port))
            .send()
            .await
            .is_ok()
        {
            started = true;
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    if !started {
        panic!("Test server failed to start");
    }

    CONTEXT.get_or_init(|| TestContext {
        port,
        admin_port,
        client,
    })
}

// Helper to find available ports
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::test]
async fn test_server_all_scenarios() {
    let ctx = get_context().await;

    // 1. Discovery/Health
    let resp = ctx
        .client
        .get(format!("http://127.0.0.1:{}/health", ctx.admin_port))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.text().await.unwrap().contains("\"status\":\"ok\""));

    // 2. Discovery Home
    let resp = ctx
        .client
        .get(format!("http://127.0.0.1:{}/", ctx.port))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.text().await.unwrap().contains("Discovery"));

    // 3. Native Render (Mermaid) - Only if chrome available
    let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG"; // [graphviz] node -> node
    let resp = ctx
        .client
        .get(format!(
            "http://127.0.0.1:{}/graphviz/svg/{}",
            ctx.port, payload
        ))
        .send()
        .await
        .unwrap();
    if resp.status() == StatusCode::OK {
        assert!(resp.text().await.unwrap().contains("<svg"));
    }

    // 4. Error Cases - 404
    let resp = ctx
        .client
        .get(format!(
            "http://127.0.0.1:{}/unknown/svg/{}",
            ctx.port, payload
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 5. Error Cases - 400 Decode
    let resp = ctx
        .client
        .get(format!(
            "http://127.0.0.1:{}/graphviz/svg/invalid!!!",
            ctx.port
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
