#[cfg(feature = "native-browser")]
use kroki_rs::config::Config;
#[cfg(feature = "native-browser")]
use kroki_rs::server;
#[cfg(feature = "native-browser")]
use reqwest::{Client, StatusCode};
#[allow(unused_imports)]
use std::sync::Arc;
#[cfg(feature = "native-browser")]
use std::time::Duration;
#[allow(unused_imports)]
use tokio::sync::Semaphore;
#[cfg(feature = "native-browser")]
use tokio::time::sleep;

#[cfg(feature = "native-browser")]
fn get_available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[cfg(feature = "native-browser")]
async fn start_test_server() -> (u16, u16) {
    let port = get_available_port();
    let admin_port = get_available_port();

    let mut config = Config::default();
    config.server.port = port;
    config.server.admin_port = admin_port;
    config.browser.pool_size = 2;

    tokio::spawn(async move {
        let _ = tracing_subscriber::fmt::try_init();
        server::run(config).await.unwrap();
    });

    let client = reqwest::Client::new();
    let mut started = false;
    for _ in 0..150 {
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
        panic!("Test server failed to start within 30 seconds");
    }

    (port, admin_port)
}

/// SANITY TEST: Executes a request through the Native Browser Backend.
/// Strictly guarded by the "native-browser" feature.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg(feature = "native-browser")]
async fn test_browser_sanity() {
    let (port, _) = start_test_server().await;
    let client = Client::new();

    let valid_mermaid_payload = {
        use base64::prelude::*;
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"graph TD;\nA-->B;").unwrap();
        let compressed = encoder.finish().unwrap();
        BASE64_URL_SAFE.encode(compressed)
    };

    let url = format!(
        "http://127.0.0.1:{}/mermaid/svg/{}",
        port, valid_mermaid_payload
    );
    let resp = client.get(&url).send().await.expect("Request failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let txt = resp.text().await.unwrap();
    assert!(txt.contains("<svg"));
}

/// LOAD TEST: Smashes the headless_chrome backend.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
#[cfg(feature = "native-browser")]
async fn test_load_native_concurrency() {
    let (port, _) = start_test_server().await;
    let client = Client::new();

    let valid_mermaid_payload = {
        use base64::prelude::*;
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"graph TD;\nA-->B;").unwrap();
        let compressed = encoder.finish().unwrap();
        BASE64_URL_SAFE.encode(compressed)
    };

    let concurrency_level = 10;
    let total_requests = 60;

    let semaphore = Arc::new(Semaphore::new(concurrency_level));
    let mut handles = Vec::new();

    for i in 0..total_requests {
        let client_clone = client.clone();
        let payload = valid_mermaid_payload.to_string();
        let sem_clone = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let url = format!("http://127.0.0.1:{}/mermaid/svg/{}", port, payload);
            let resp = client_clone.get(&url).send().await.expect("Request failed");

            if resp.status() != StatusCode::OK {
                panic!(
                    "Native worker failed at request {} with status {}",
                    i,
                    resp.status()
                );
            }
            let txt = resp.text().await.unwrap();
            assert!(txt.contains("<svg"));
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
