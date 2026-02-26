use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
pub use metrics_exporter_prometheus::PrometheusHandle;

use std::sync::OnceLock;

/// A global handle to the Prometheus exporter.
static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus metrics exporter.
///
/// This registers the recorder and defines all core metrics (counters, histograms, gauges)
/// according to ADR 0006.
///
/// Returns a handle that can be used to scrape metrics if the endpoint is enabled.
/// This function is idempotent and safe to call multiple times (e.g. in integration tests).
pub fn init_metrics() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            let builder = PrometheusBuilder::new();

            // Configure histogram buckets as per ADR 0006
            let buckets = [0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
            let handle = builder
                .set_buckets(&buckets)
                .expect("Failed to set metrics buckets")
                .install_recorder()
                .expect("Failed to install Prometheus recorder");

            describe_metrics();

            // Record cold start to ensure metrics page has data immediately
            counter!("kroki_server_starts_total").increment(1);

            handle
        })
        .clone()
}

fn describe_metrics() {
    describe_counter!("kroki_requests_total", "Total number of diagram requests");
    describe_histogram!(
        "kroki_request_duration_seconds",
        "Total duration of the diagram request in seconds"
    );
    describe_counter!(
        "kroki_rendering_errors_total",
        "Total number of rendering errors"
    );
    describe_histogram!(
        "kroki_payload_size_bytes",
        "Input payload size distribution in bytes"
    );
    describe_histogram!(
        "kroki_conversion_time_seconds",
        "Time spent in the diagram provider rendering"
    );
    describe_gauge!(
        "kroki_active_connections",
        "Number of currently active concurrent requests"
    );
    describe_gauge!(
        "kroki_circuit_breaker_state",
        "Current state of the circuit breaker (0=closed, 1=open, 2=half-open)"
    );
    describe_counter!(
        "kroki_server_starts_total",
        "Total number of server cold starts"
    );
}

/// Helper for recording common metrics in API and Admin handlers.
///
/// This provides a static interface to record requests, durations, errors,
/// and circuit breaker states without needing to pass around recorder handles.
pub struct Metrics;

impl Metrics {
    /// Increments the total request count for a given provider and format.
    pub fn increment_requests(provider: &str, format: &str) {
        counter!("kroki_requests_total", "provider" => provider.to_string(), "format" => format.to_string()).increment(1);
    }

    /// Records the end-to-end duration of a request.
    pub fn record_duration(provider: &str, format: &str, seconds: f64) {
        histogram!("kroki_request_duration_seconds", "provider" => provider.to_string(), "format" => format.to_string()).record(seconds);
    }

    /// Records a rendering error, categorized by error kind.
    pub fn increment_errors(provider: &str, format: &str, error_kind: &str) {
        counter!("kroki_rendering_errors_total",
            "provider" => provider.to_string(),
            "format" => format.to_string(),
            "error_kind" => error_kind.to_string()
        )
        .increment(1);
    }

    /// Records the size of the input payload (compressed/encoded source).
    pub fn record_payload_size(provider: &str, format: &str, bytes: f64) {
        histogram!("kroki_payload_size_bytes", "provider" => provider.to_string(), "format" => format.to_string()).record(bytes);
    }

    /// Records the pure conversion time (excluding server overhead).
    pub fn record_conversion_time(provider: &str, format: &str, seconds: f64) {
        histogram!("kroki_conversion_time_seconds", "provider" => provider.to_string(), "format" => format.to_string()).record(seconds);
    }

    /// Updates the gauge for currently active concurrent requests.
    pub fn set_active_connections(provider: &str, format: &str, count: f64) {
        gauge!("kroki_active_connections", "provider" => provider.to_string(), "format" => format.to_string()).set(count);
    }

    /// Updates the gauge for a provider's circuit breaker state (0=Closed, 1=Open, 2=HalfOpen).
    pub fn set_circuit_breaker_state(provider: &str, state: f64) {
        gauge!("kroki_circuit_breaker_state", "provider" => provider.to_string()).set(state);
    }
}
