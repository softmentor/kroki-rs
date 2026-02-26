use kroki_rs::config::{ApiKeyEntry, Config};
use kroki_rs::server;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

async fn start_test_server(mut config: Config) -> (u16, u16) {
    let port = get_available_port();
    let admin_port = get_available_port();
    config.server.port = port;
    config.server.admin_port = admin_port;
    config.server.timeout_ms = 30000; // 30s for integration tests

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
async fn test_api_key_authentication() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut config = Config::default();
        config.server.auth.enabled = true;
        config.server.auth.api_keys.push(ApiKeyEntry {
            key: "valid-key-123".to_string(),
            label: "test-user".to_string(),
            rate_limit: None,
        });

        let (port, _) = start_test_server(config).await;
        let client = Client::new();
        let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG"; // graphviz node -> node

        // 1. Missing Key -> 401
        let resp = client
            .get(format!(
                "http://127.0.0.1:{}/graphviz/svg/{}",
                port, payload
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(resp.text().await.unwrap().contains("Missing API key"));

        // 2. Invalid Key -> 401
        let resp = client
            .get(format!(
                "http://127.0.0.1:{}/graphviz/svg/{}",
                port, payload
            ))
            .header("X-Api-Key", "wrong-key")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(resp.text().await.unwrap().contains("Invalid API key"));

        // 3. Valid Key -> 200 (if dot available)
        let resp = client
            .get(format!(
                "http://127.0.0.1:{}/graphviz/svg/{}",
                port, payload
            ))
            .header("X-Api-Key", "valid-key-123")
            .send()
            .await
            .unwrap();

        // We check for 404 or 200 depending on if 'dot' is actually in the test environment PATH
        // But since it's an integration test, we assume if dot is missing it might be 404 or ToolNotFound.
        // The key is that it's NOT 401.
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    })
    .await
    .expect("test_api_key_authentication timed out");
}

#[tokio::test]
async fn test_admin_basic_auth() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut config = Config::default();
        config.server.auth.enabled = true;
        let hash = bcrypt::hash("admin123", 4).unwrap();
        config.server.auth.admin_password_hash = Some(hash);

        let (_, admin_port) = start_test_server(config).await;
        let client = Client::new();

        // 1. No Auth -> 401
        let resp = client
            .get(format!("http://127.0.0.1:{}/metrics", admin_port))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // 2. Wrong Credentials -> 401
        let resp = client
            .get(format!("http://127.0.0.1:{}/metrics", admin_port))
            .basic_auth("admin", Some("wrong"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // 3. Valid Credentials -> 200
        let resp = client
            .get(format!("http://127.0.0.1:{}/metrics", admin_port))
            .basic_auth("admin", Some("admin123"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    })
    .await
    .expect("test_admin_basic_auth timed out");
}

#[tokio::test]
async fn test_rate_limiting() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut config = Config::default();
        config.server.rate_limit.enabled = true;
        config.server.rate_limit.requests_per_second = 1;
        config.server.rate_limit.burst_size = 1;

        let (port, _) = start_test_server(config).await;
        let client = Client::new();
        let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";

        let mut got_429 = false;
        for _ in 0..10 {
            let resp = client
                .get(format!(
                    "http://127.0.0.1:{}/graphviz/svg/{}",
                    port, payload
                ))
                .send()
                .await
                .unwrap();
            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                got_429 = true;
                assert!(resp.headers().contains_key("retry-after"));
                break;
            }
        }
        assert!(got_429, "Expected at least one 429 TOO_MANY_REQUESTS");
    })
    .await
    .expect("test_rate_limiting timed out");
}

#[tokio::test]
async fn test_circuit_breaker() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut config = Config::default();
        config.server.circuit_breaker.enabled = true;
        config.server.circuit_breaker.failure_threshold = 2;
        // Set a long timeout so it stays open during our test
        config.server.circuit_breaker.reset_timeout_secs = 60;

        // Use a tool that exists but fails to trigger recorded failures
        // '/usr/bin/false' exists on most unix systems and returns exit code 1
        config.graphviz.bin_path = Some("/usr/bin/false".to_string());

        let (port, _) = start_test_server(config).await;
        let client = Client::new();
        let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";

        // Failure 1 & 2
        for _ in 0..2 {
            let resp = client
                .get(format!(
                    "http://127.0.0.1:{}/graphviz/svg/{}",
                    port, payload
                ))
                .send()
                .await
                .unwrap();
            // It failed because tool is /usr/bin/false, should return 500
            assert_ne!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        }

        // Request 3 -> 503 (Circuit Open)
        let resp = client
            .get(format!(
                "http://127.0.0.1:{}/graphviz/svg/{}",
                port, payload
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    })
    .await
    .expect("test_circuit_breaker timed out");
}

#[tokio::test]
async fn test_prometheus_metrics() {
    tokio::time::timeout(Duration::from_secs(30), async {
        let mut config = Config::default();
        config.server.metrics.enabled = true;
        let (port, admin_port) = start_test_server(config).await;
        let client = Client::new();

        // Make a request to generate metrics
        let payload = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";
        client
            .get(format!(
                "http://127.0.0.1:{}/graphviz/svg/{}",
                port, payload
            ))
            .send()
            .await
            .unwrap();

        let resp = client
            .get(format!("http://127.0.0.1:{}/metrics", admin_port))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.text().await.unwrap();
        // Check for some standard prometheus metric markers
        assert!(body.contains("kroki_requests_total"));
    })
    .await
    .expect("test_prometheus_metrics timed out");
}
