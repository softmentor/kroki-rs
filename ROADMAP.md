---
title: Project Roadmap
label: kroki-rs.roadmap
---
# 🗺️ Kroki-rs Roadmap

This document outlines the planned internal improvements and future features for **Kroki-rs**. We aim to make this the most efficient and reliable unified diagramming service using the Rust ecosystem.

## 🟢 v0.0.1: Initial Release (Completed)
- [x] Core diagram generation engine
- [x] CLI and Server modes
- [x] Major diagram providers (Graphviz, Mermaid, PlantUML, D2, BPMN, Wavedrom, Vega, Ditaa)

## � v0.0.2: Performance & Expansion (Completed)
- [x] **Excalidraw Support**: Implement the remaining major provider.
- [x] **Local Caching Layer**: Add optional filesystem-based caching of rendered SVGs.
- [x] **Parallel Batch Conversion**: CLI support for converting entire directories.
- [x] **WebP Format Support**: Centralized conversion of SVG/PNG to high-fidelity WebP.
- [x] **Just-in-Time Font Loading**: Hashed font caching for high-fidelity rasterization (ADR 0002).
- [x] **Async Refactoring**: Fully non-blocking subprocess execution via Tokio (ADR 0003).
- [x] **Adaptive Timeouts**: Dynamic subprocess management to mitigate ReDoS and hangs.
- [x] **Documentation Standardization**: Full MyST label system and path-independent navigation.

## � v0.0.3: Stability & Core Polish (Current)
- [ ] **Custom Path Configuration**: Allow setting tool paths (like `dot` or `mmdc`) via a centralized `kroki.toml`.
- [ ] **Enhanced Error Messaging**: Return more descriptive rendering errors (STDERR capture) to the client.
- [ ] **Production Multi-arch Images**: Provide OCI-compliant, streamlined Docker images.
- [ ] **Health Check API**: Endpoint for container orchestration and uptime monitoring.

## 🟠 v0.0.4: Server Features & Observability
- [ ] **API Key Authentication**: Simple token-based security for the `serve` endpoint.
- [ ] **Prometheus Metrics**: Export rendering stats (request count, latency per engine, failure rates).
- [ ] **Custom Plugin API**: Allow users to add their own "DiagramProvider".
- [ ] **CNCF Standards Alignment**: Adopt **CloudEvents** for rendering triggers and **OpenTelemetry** for observability.

## � v0.1.0: The Unified Vision
- [ ] **WASM Rendering**: Investigate moving some engines (like Mermaid) to a JS-in-Rust evaluator (e.g., `deno_core` or `rustyscript`) for truly zero-dependency builds.
- [ ] **Live Preview Web UI**: A simple management dashboard to test conversions and view server health.
- [ ] **Edge Deployment**: Optimize for Cloudflare Workers / Fastly Compute@Edge contexts.

---
> Have a suggestion? Feel free to open an [Issue](https://github.com/softmentor/kroki-rs/issues) or a [Pull Request](https://github.com/softmentor/kroki-rs/pulls).
