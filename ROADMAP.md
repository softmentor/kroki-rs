# 🗺️ Kroki-rs Roadmap

This document outlines the planned internal improvements and future features for **Kroki-rs**. We aim to make this the most efficient and reliable unified diagramming service in the Rust ecosystem.

## 🟢 v0.1.x: Stability & Core Polish
- [ ] **Excalidraw Support**: Implement the remaining major provider.
- [ ] **Custom Path Configuration**: Allow setting tool paths (like `dot` or `mmdc`) via a centralized `kroki.toml`.
- [ ] **Enhanced Error Messaging**: Return more descriptive rendering errors (STDERR capture) to the client.

## 🟡 v0.2.x: Performance & Efficiency
- [ ] **Local Caching Layer**: Add optional SQLite or filesystem-based caching of rendered SVGs to avoid redundant subprocess calls.
- [ ] **Parallel Batch Conversion**: CLI support for converting entire directories of diagrams in one command.
- [ ] **Streamlined Docker Image**: A multi-stage build that includes all necessary CLI tools in a minimal (Alpine/Distroless) image.

## 🟠 v0.3.x: Server Features
- [ ] **API Key Authentication**: Simple token-based security for the `serve` endpoint.
- [ ] **Prometheus Metrics**: Export rendering stats (request count, latency per engine, failure rates).
- [ ] **Custom Plugin API**: Allow users to add their own "DiagramProvider" via a simple configuration or shared library.

## 🔴 v1.0.0 & Beyond: The Unified Vision
- [ ] **WASM Rendering**: Investigate moving some engines (like Mermaid) to a JS-in-Rust evaluator (e.g., `deno_core` or `rustyscript`) for truly zero-dependency builds.
- [ ] **Live Preview Web UI**: A simple management dashboard to test conversions and view server health.
- [ ] **Edge Deployment**: Optimize for Cloudflare Workers / Fastly Compute@Edge contexts.

---
> Have a suggestion? Feel free to open an [Issue](https://github.com/softmentor/kroki-rs/issues) or a [Pull Request](https://github.com/softmentor/kroki-rs/pulls).
