---
title: Advanced Features
label: kroki-rs.user-guide.features
---
# Features

Kroki-rs is packed with features designed for speed, reliability, and ease of use.

## Unified Diagram API
- **Single Endpoint**: Access 10+ diagramming tools through a consistent HTTP interface.
- **CLI & Server**: Switch between one-shot CLI conversions and a persistent web service.
- **Drop-in Support**: Fully compatible with the original Kroki API specification for GET requests.

## Performance & Efficiency
- **Native Execution**: Leverages industry-standard CLI tools for accurate rendering, running directly on your host to avoid the overhead of massive Docker images.
- **Rust Core**: Built with Axum and Tokio for high concurrency, maximum efficiency, and blazing-fast response times.
- **Lazy Discovery**: Tools are only discovered and initialized when needed, keeping startup times minimal.

## Production Readiness
- **Layered Middleware**: Configurable authentication (API keys), per-provider circuit breakers, and token-bucket rate limiting to protect your infrastructure.
- **Observability**: Built-in Prometheus exporter for real-time tracking of request volume, latency histograms, error rates, and circuit breaker states.
- **Admin Dashboard**: A unified dashboard for health monitoring, system status, and live metrics visualization.
- **Service Discovery**: Interactive home page at the service root to help team members find endpoints and supported features.

## Extensibility
- **Custom Plugin API**: Register your own diagramming tools without modifying the core. Use a simple subprocess protocol and templating to extend Kroki-rs with any command-line renderer.

## Developer Experience
- **Structured Logging**: High-quality structured logging using `tracing`. Control log levels via the `RUST_LOG` environment variable for deep debugging.
- **Comprehensive Docs**: Integrated MyST-based documentation and Rustdoc API references.
- **Robust Tooling**: A professional `Makefile` providing a unified interface for build, test, lint, and distribution.
- **CI/CD Ready**: Automated GitHub Actions for deploying docs to GitHub Pages and creating tagged releases with signed artifacts.

## Supported Formats
- **Mermaid**: Flowcharts, Sequences, Gantt, etc.
- **Graphviz**: DOT language visualization.
- **D2**: Modern, declarative diagramming.
- **Vega/Vega-Lite**: Data visualization.
- **And more**: BPMN, WaveDrom, Ditaa.
