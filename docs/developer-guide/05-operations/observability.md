---
title: Observability & Monitoring
label: kroki-rs.developer-guide.observability
---

# Observability & Monitoring

Kroki-rs provides structured logging and metrics to ensure operational visibility across local and production environments.

## Logging Strategy

We use the `tracing` crate for asynchronous, structured logging.

- **Human Readable**: Default output is optimized for terminal readability.
- **Contextual**: Logs include provider names, payload sizes, and precise timing for rendering steps.

### Log Levels
| Level | Usage |
| :--- | :--- |
| `ERROR` | Fatal failures, process crashes, or invalid configurations. |
| `WARN` | Recoverable errors (e.g., cache miss with fallback). |
| `INFO` | Major lifecycle events (Server start, Provider discovery). |
| `DEBUG` | Detailed internal state and binary path resolution. |
| `TRACE` | High-frequency events (Per-byte decoding steps). |

## Metrics & Dashboard (Admin Port)

The server exposes a dedicated **Admin server** on port `8081` (default) for observability and orchestration.

### Operations Dashboard
| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/health` | GET | Basic health check for server and browser pool. |
| `/metrics` | GET | Prometheus-style metrics (Renders, Latency, Errors). |

### Real-time Pool Monitoring
The `/health` endpoint also returns JSON telemetry for the browser pool:
- `active_workers`: Number of tabs currently rendering.
- `spare_workers`: Idle tabs ready for immediate use.
- `pending_tasks`: Queue depth for diagram requests.

## Enabling Observability
Use the `-d` or `--debug` flag with `dflow` to see the full diagnostic output:
```bash
./dflow develop --debug
```
