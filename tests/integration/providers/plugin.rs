use kroki_rs::config::{Config, PluginConfig};
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

async fn start_test_server(mut config: Config) -> (u16, u16) {
    let port = get_available_port();
    let admin_port = get_available_port();
    config.server.port = port;
    config.server.admin_port = admin_port;

    tokio::spawn(async move {
        let _ = server::run(config).await;
    });

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
        sleep(Duration::from_millis(100)).await;
    }

    if !started {
        panic!("Test server failed to start");
    }

    (port, admin_port)
}

fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::test]
async fn test_custom_plugin_registration_and_execution() {
    let mut config = Config::default();

    // Define a "mock" plugin using 'echo'
    config.plugins.push(PluginConfig {
        name: "test-echo".to_string(),
        command: "echo".to_string(),
        args: vec!["rendered-by-plugin-{format}".to_string()],
        stdin: false,
        formats: vec!["svg".to_string()],
        timeout_ms: Some(2000),
    });

    let (port, _) = start_test_server(config).await;
    let client = Client::new();
    let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG"; // Dummy payload

    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/test-echo/svg/{}",
            port, payload
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert_eq!(body.trim(), "rendered-by-plugin-svg");
}

#[tokio::test]
async fn test_plugin_with_stdin() {
    let mut config = Config::default();

    // Use 'cat' to echo back the source
    config.plugins.push(PluginConfig {
        name: "test-cat".to_string(),
        command: "cat".to_string(),
        args: vec![],
        stdin: true,
        formats: vec!["svg".to_string()],
        timeout_ms: Some(2000),
    });

    let (port, _) = start_test_server(config).await;
    let client = Client::new();

    // "digraph G { Hello -> World }" in zlib/base64
    let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";

    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/test-cat/svg/{}",
            port, payload
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    // 'cat' should just return the decompressed source
    assert_eq!(body.trim(), "digraph G { Hello -> World }");
}
