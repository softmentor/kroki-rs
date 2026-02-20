use kroki_rs::config::Config;
use std::env;

#[test]
fn test_config_env_overrides() {
    // Save current env vars to avoid mutating global state weirdly
    // Actually rust runs tests in parallel so env manipulation can be flaky
    // We will namespace the test env vars uniquely if possible, or just lock them

    // Set custom env variables
    env::set_var("KROKI_PORT", "9999");
    env::set_var("KROKI_ADMIN_PORT", "9998");
    env::set_var("KROKI_TIMEOUT", "12345");
    env::set_var("KROKI_MAX_INPUT_SIZE", "5000");
    env::set_var("KROKI_MERMAID_BIN", "/custom/mmdc");

    // Load config natively with None to bypass file and just use Env
    let config = Config::load(None).expect("Failed to load config");

    assert_eq!(config.server.port, 9999);
    assert_eq!(config.server.admin_port, 9998);
    assert_eq!(config.server.timeout_ms, 12345);
    assert_eq!(config.server.max_input_size, 5000);
    assert_eq!(config.mermaid.bin_path, Some("/custom/mmdc".to_string()));

    // Cleanup
    env::remove_var("KROKI_PORT");
    env::remove_var("KROKI_ADMIN_PORT");
    env::remove_var("KROKI_TIMEOUT");
    env::remove_var("KROKI_MAX_INPUT_SIZE");
    env::remove_var("KROKI_MERMAID_BIN");
}
