---
title: "ADR 0006: Per-Provider Observability & Metrics"
label: kroki-rs.adr.0006
---

## Context
Production deployments require visibility into per-provider performance to diagnose bottlenecks, set SLOs, and inform capacity planning. Operators need to answer: "Which diagram type is slowest?", "Which provider fails most?", "What's the P99 latency?"

## Decision

### Prometheus Metrics (per diagram provider)

All metrics are tagged with `{provider, format}` dimensions for granular breakdowns:

| Metric | Type | Notes |
|--------|------|-------|
| `kroki_requests_total` | Counter | Per provider × format |
| `kroki_request_duration_seconds` | Histogram | p75, p90, p99 buckets |
| `kroki_rendering_errors_total` | Counter | Tagged with `error_kind` |
| `kroki_payload_size_bytes` | Histogram | Input payload size distribution |
| `kroki_conversion_time_seconds` | Histogram | Provider-internal render time |
| `kroki_active_connections` | Gauge | Current concurrent requests |
| `kroki_circuit_breaker_state` | Gauge | 0=closed, 1=open, 2=half-open |

### Implementation
- Use `metrics` + `metrics-exporter-prometheus` crates (lightweight, no OTel overhead by default).
- Metrics collection is always-on when configured. The `/metrics` export endpoint on the admin port is **optional** (`server.metrics.export_endpoint = true`).
- Histogram buckets are tuned for diagram rendering: `[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]` seconds.

### OpenTelemetry (Optional)
- Behind the `otel` Cargo feature flag.
- Bridges existing `tracing` spans to OTLP via `tracing-opentelemetry`.
- OTLP endpoint configurable via `kroki.toml` (`server.telemetry.otlp_endpoint`) **or** `OTEL_EXPORTER_OTLP_ENDPOINT` env var.

### Configuration
```toml
[server.metrics]
enabled = true
export_endpoint = false

[server.telemetry]
enabled = false
otlp_endpoint = ""
```

## Consequences
- **Positive**: Operators get deep visibility into per-provider performance without any external tooling.
- **Positive**: Optional export endpoint means zero overhead when not scraping.
- **Positive**: OTel integration reuses existing `tracing` instrumentation — no code duplication.
- **Negative**: Histogram memory footprint grows linearly with number of providers × formats.
