use kroki_rs::config::Config;
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

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
    // VERY low TTL to force the pool to eagerly recycle BrowserContexts
    config.browser.pool_size = 2;
    config.browser.context_ttl_requests = 10;

    tokio::spawn(async move {
        // Ensure logs are visible in tests if RUST_LOG is set
        let _ = tracing_subscriber::fmt::try_init();
        server::run(config).await.unwrap();
    });

    let client = reqwest::Client::new();
    let mut started = false;
    for i in 0..150 {
        if client
            .get(format!("http://127.0.0.1:{}/health", admin_port))
            .send()
            .await
            .is_ok()
        {
            started = true;
            break;
        }
        if i % 10 == 0 {
            println!("Waiting for test server... {}s", i / 5);
        }
        sleep(Duration::from_millis(200)).await;
    }

    if !started {
        panic!("Test server failed to start within 30 seconds");
    }

    (port, admin_port)
}

/// SANITY TEST: Executes a single successful request through the BrowserManager
/// to ensure basic Playwright/Node integration is functional. Fast enough for CI.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_playwright_sanity() {
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
    assert!(txt.contains("<svg")); // ensure actual svg generated
}

/// LOAD TEST: Smashes the Playwright pool with concurrent requests to verify
/// resource limits and TTL recycling.
/// Run locally with: `cargo test --test integration test_load_playwright_concurrency -- --ignored`
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn test_load_playwright_concurrency() {
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

    // Fire 60 concurrent requests at the pool of size 2.
    // With a TTL of 10, this guarantees the pool will have to destroy and recreate
    // contexts actively while under load.
    let concurrency_level = 10;
    let total_requests = 60;

    let semaphore = Arc::new(Semaphore::new(concurrency_level));
    let mut handles = Vec::new();

    for _ in 0..total_requests {
        let client_clone = client.clone();
        let payload = valid_mermaid_payload.to_string();
        let sem_clone = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let url = format!("http://127.0.0.1:{}/mermaid/svg/{}", port, payload);
            let resp = client_clone.get(&url).send().await.expect("Request failed");
            if resp.status() != StatusCode::OK {
                let status = resp.status();
                let txt = resp.text().await.unwrap_or_default();
                panic!("Worker failed with status {}: {}", status, txt);
            }
            let txt = resp.text().await.unwrap();
            assert!(txt.contains("<svg")); // ensure actual svg generated
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

/// NATIVE LOAD TEST: Smashes the headless_chrome backend with concurrent requests.
/// Verification for the new 0.0.5 native engine.
/// Run locally with: `cargo test --test integration test_load_native_concurrency -- --ignored`
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
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

    // Fire 60 concurrent requests.
    // Native backend (headless_chrome) uses on-demand tabs.
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
                let status = resp.status();
                let txt = resp.text().await.unwrap_or_default();
                panic!(
                    "Native worker failed at request {} with status {}: {}",
                    i, status, txt
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
