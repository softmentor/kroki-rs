---
title: Docker Deployment Guide
label: kroki-rs.user-guide.docker-deployment
---

# Docker Deployment Guide

The official **Kroki-rs** Docker image is the recommended way to deploy the service in production. It packages all native dependencies (Chromium, Node.js, Graphviz, D2) in a highly-optimized, multi-arch container.

## 1. Quick Start (Production)

To run the latest stable version of Kroki-rs:

```bash
docker run -d \
  --name kroki-rs \
  -p 8000:8000 \
  -p 8081:8081 \
  --restart unless-stopped \
  ghcr.io/softmentor/kroki-rs:latest
```

### Port Mapping
| Port | Purpose | Description |
| :--- | :--- | :--- |
| **8000** | **Main API** | Server for diagram rendering and conversion. |
| **8081** | **Admin Port** | Internal port for health checks, metrics, and monitoring. |

## 2. Operations & Monitoring

Kroki-rs separates operational traffic (metrics/health) from rendering traffic to ensure monitoring remains responsive even under heavy load.

### Health Checks
The `/health` endpoint on the Admin port (`8081`) returns a 200 OK status if the server and its internal browser pool are operational.

```bash
curl http://localhost:8081/health
```

### Metrics & Observability
The Admin port also provides real-time insights into the service's health:
- **Browser Pool Status**: Active, spare, and pending browser worker counts.
- **Memory Consumption**: Track the footprint of the Rust binary and headless Chrome processes.

## 3. Configuration & Tuning

### Environment Variables
You can tune the service behavior using environment variables passed to the container:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `KROKI_PORT` | `8000` | The primary API port. |
| `KROKI_ADMIN_PORT` | `8081` | The administrative monitoring port. |

### Mounting Configuration
For advanced tuning (like custom fonts or timeout policies), mount a `kroki.toml` file:

```bash
docker run -d \
  -v $(pwd)/kroki.toml:/app/kroki.toml \
  -p 8000:8000 \
  ghcr.io/softmentor/kroki-rs:latest
```

## 4. Debugging & Troubleshooting

### Container Logs
Inspect logs from both the Rust server and the background browser workers:
```bash
docker logs -f kroki-rs
```

### Resource Constraints
Renderings diagrams like Mermaid can be CPU and memory intensive. We recommend the following minimum resources for a production instance:
- **Memory**: 1GB (2GB+ recommended if using complex Mermaid/BPMN).
- **CPU**: 2 Cores.

### Common Issues
- **Font Rendering**: If symbols appear as blocks, ensure your `kroki.toml` points to correct font URLs or that you've mounted a local font directory if custom-built.
- **Timeouts**: If large diagrams fail, increase the `diagrams.timeout_base_ms` in your configuration.
