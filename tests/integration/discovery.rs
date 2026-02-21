use kroki_rs::config::Config;
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

async fn start_test_server() -> u16 {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let admin_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let mut config = Config::default();
    config.server.port = port;
    config.server.admin_port = admin_port;

    tokio::spawn(async move {
        server::run(config).await.unwrap();
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
        sleep(Duration::from_millis(200)).await;
    }

    if !started {
        panic!("Test server failed to start within 10 seconds");
    }

    port
}

#[tokio::test]
async fn test_discovery_page() {
    let port = start_test_server().await;
    let client = Client::new();

    let url = format!("http://127.0.0.1:{}", port);
    let resp = client.get(&url).send().await.expect("Request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    let html = resp.text().await.unwrap();

    // Verify core elements of the discovery page
    assert!(html.contains("<title>Kroki-rs | Discovery</title>"));
    assert!(html.contains("<h1>Kroki-rs</h1>"));
    assert!(html.contains("Service Status"));
    assert!(html.contains("Endpoints"));
    assert!(html.contains("Available Providers"));
}
