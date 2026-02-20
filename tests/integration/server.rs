use kroki_rs::config::Config;
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

// Helper to find available ports
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn start_test_server() -> (u16, u16) {
    let port = get_available_port();
    let admin_port = get_available_port();

    let mut config = Config::default();
    config.server.port = port;
    config.server.admin_port = admin_port;
    // Lower max input size to easily test 413 Payload Too Large
    config.server.max_input_size = 50;

    // Find paths so things work if installed
    config.graphviz.bin_path = Some(
        which::which("dot")
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );
    // Deliberately point one tool to a non-existent binary to test 503
    config.ditaa.bin_path = Some("/path/to/nowhere/ditaa".to_string());

    tokio::spawn(async move {
        server::run(config).await.unwrap();
    });

    // Wait up to 10 seconds for the server to start (Playwright worker takes a moment)
    let client = reqwest::Client::new();
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

    (port, admin_port)
}

#[tokio::test]
async fn test_server_health_and_admin() {
    let (_port, admin_port) = start_test_server().await;
    let client = Client::new();

    let resp = client
        .get(format!("http://127.0.0.1:{}/health", admin_port))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.text().await.unwrap();
    assert!(body.contains("\"status\":\"ok\""));

    let resp_dash = client
        .get(format!("http://127.0.0.1:{}/", admin_port))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(resp_dash.status(), StatusCode::OK);
    let html = resp_dash.text().await.unwrap();
    assert!(html.contains("Kroki-rs"));
}

#[tokio::test]
async fn test_server_diagram_endpoints() {
    let (port, _admin_port) = start_test_server().await;
    let client = Client::new();

    // 1. Root string
    let resp = client
        .get(format!("http://127.0.0.1:{}/", port))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.text().await.unwrap().contains("Kroki-rs is running"));

    // 2. Unsupported format -> 400 Bad Request
    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/graphviz/invalidfmt/eNpLyUwvSizm5TIGAAWDAY0=",
            port
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 3. Invalid Base64/Decode -> 400 Bad Request
    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/graphviz/svg/invalid_base64_",
            port
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let valid_payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";

    // 4. Unknown tool -> 404 Not Found
    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/unknown_tool/svg/{}",
            port, valid_payload
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 5. Tool configured with a bad binary path -> 503 Service Unavailable / 404 Not Found
    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/ditaa/png/{}",
            port, valid_payload
        ))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::SERVICE_UNAVAILABLE || resp.status() == StatusCode::NOT_FOUND
    );

    // 6. Payload too large -> 413
    // Simulate by manually creating a large valid compressed payload
    let large_payload = {
        use base64::prelude::*;
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all("A".repeat(100).as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        BASE64_URL_SAFE.encode(compressed)
    };

    let resp = client
        .get(format!(
            "http://127.0.0.1:{}/graphviz/svg/{}",
            port, large_payload
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
