# 🗺️ Kroki-rs Roadmap

This document outlines the planned internal improvements and future features for **Kroki-rs**. We aim to make this the most efficient and reliable unified diagramming service using the Rust ecosystem.

## 🟢 v0.0.1: Initial Release (Completed)
- [x] Core diagram generation engine
- [x] CLI and Server modes
- [x] Major diagram providers (Graphviz, Mermaid, PlantUML, D2, BPMN, Wavedrom, Vega, Ditaa)

## 🟡 v0.0.2: Performance & Expansion (Current)
- [x] **Excalidraw Support**: Implement the remaining major provider.
- [x] **Local Caching Layer**: Add optional filesystem-based caching of rendered SVGs to avoid redundant subprocess calls.
- [x] **Parallel Batch Conversion**: CLI support for converting entire directories of diagrams in one command.
- [x] **Integration Testing Improvements**: Dedicated test coverage for individual providers.
- [x] **WebP Format Support**: Centralized conversion of SVG/PNG to high-fidelity WebP using `resvg` and `image` crates.
- [x] **High-Level Configuration**: Ability to tune WebP quality settings via CLI flags or `kroki.toml`.

## 🟠 v0.0.3: Stability & Core Polish
- [ ] **Custom Path Configuration**: Allow setting tool paths (like `dot` or `mmdc`) via a centralized `kroki.toml`.
- [ ] **Custom Font Loading**: Provide configurations to load specific `.ttf` files (like Google Fonts) into the local `fontdb` context for high-fidelity WebP and SVG rasterizations.
- [ ] **Enhanced Error Messaging**: Return more descriptive rendering errors (STDERR capture) to the client.
- [ ] **Optional Production Targets**: Provide OCI-compliant, streamlined Docker/Distroless images for containerized environments (K8s, ECS).

## � v0.0.4: Server Features & Observability
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
