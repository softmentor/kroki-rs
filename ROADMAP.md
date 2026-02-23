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

## 🟢 v0.0.2: Performance & Expansion (Completed)
- [x] **Excalidraw Support**: Implement the remaining major provider.
- [x] **Local Caching Layer**: Add optional filesystem-based caching of rendered SVGs.
- [x] **Parallel Batch Conversion**: CLI support for converting entire directories.
- [x] **WebP Format Support**: Centralized conversion of SVG/PNG to high-fidelity WebP.
- [x] **Just-in-Time Font Loading**: Hashed font caching for high-fidelity rasterization (ADR 0002).
- [x] **Async Refactoring**: Fully non-blocking subprocess execution via Tokio (ADR 0003).
- [x] **Adaptive Timeouts**: Dynamic subprocess management to mitigate ReDoS and hangs.
- [x] **Custom Path Configuration**: Support setting tool paths via `kroki.toml` or CLI overrides.
- [x] **Enhanced Error Messaging**: Captured STDERR on timeout and improved decode error granularity.
- [x] **Documentation Standardization**: Full MyST label system and path-independent navigation.
- [x] **Tech Debt Remediation**: Resolved 28 identified code quality and robustness items.

## 🟢 v0.0.3: Stability & Production Polish (Completed)
- [x] **Structured DiagramError Enum**: Implement a typed error system across all providers (TD-15).
- [x] **Migration to Playwright**: For avoid flaky implementation of bpmn, mmdc etc. which rely on Puppeteer/Chromium. Explore alternatives.
- [x] **Production Multi-arch Images**: Provide OCI-compliant, streamlined Docker images.
- [x] **Health Check API**: Endpoint for container orchestration and uptime monitoring.
- [x] **Configuration Priority Pattern**: Establish a clear hierarchy for global vs. local settings (TD-07).

## � v0.0.4: Server Production Enablement (Completed)

Incremental production-grade hardening of the existing server architecture. All features are config-gated and enabled by default, supporting a "dev mode" for fast local debugging.

### Authentication & Security
- [x] **API Key Authentication**: Token-based security for the `serve` endpoint with per-key rate limits (ADR 0005).
- [ ] **OAuth Support** (optional): OAuth2 token validation for external identity providers.
- [x] **Admin Authentication**: Password-based (bcrypt encrypted) for the admin dashboard.
- [ ] **TLS Support** (optional): Serve over HTTPS via `axum-server` with `rustls` (behind `tls` feature flag).

### Observability
- [x] **Per-Provider Prometheus Metrics**: Request count, duration, payload size, conversion time, error type — all per diagram provider (ADR 0006).
- [x] **Metrics Export Endpoint**: `/metrics` on admin port for Prometheus scraping.
- [ ] **OpenTelemetry Integration** (optional): `tracing-opentelemetry` bridge with OTLP exporter (behind `otel` feature flag).

### Server Hardening
- [x] **Rate Limiting**: Per-IP token-bucket rate limiter with configurable burst size.
- [x] **Circuit Breaker**: Per-provider circuit breaker (Closed → Open → Half-Open) to prevent cascading failures.

### Extensibility
- [x] **Custom Plugin API**: Subprocess-based plugin protocol in the core library (ADR 0007).

### Discovery & Admin
- [x] **Discovery Home Page**: Interactive dashboard at the root URL aiding endpoint discovery.
- [x] **Enhanced Admin API**: `/health`, `/metrics`, `/config` (sanitized), `/providers` endpoints.
- [x] **Admin Dashboard**: Live health indicators, request volume, circuit breaker states per provider.

## 🟢 v0.0.5: Rust-Native Core — Eliminate Runtime Dependencies (Completed)

Major internal architecture shift. Remove Node.js and Java runtime dependencies in favor of pure-Rust alternatives, enabling single-binary deployment.

### Browser Automation (Eliminate Node.js)
- [x] **Rust Browser Automation Experiment**: Evaluate `fantoccini` (WebDriver), `headless_chrome` (CDP), and `chromiumoxide` (CDP) as replacements for Node.js + Playwright (ADR 0008).
- [x] **Native Backend**: Implement pure-Rust browser automation using `headless_chrome` with automated fallback to the legacy Playwright backend.

### PlantUML without Java

### Design & Architecture
- [x] **BrowserBackend Trait**: Abstract browser implementations behind a trait for future crate separation (ADR 0008.1).
- [x] **Composite CI/CD**: Standardized setup across workflows using a reusable composite action (ADR 0010).

## 🔮 v0.0.6: Modular Crate Workspace

Split into multi-crate Cargo workspace for maximum reuse (VS Code plugins, desktop apps, embedded systems).

```
kroki-rs/
├── Cargo.toml              # workspace root
├── crates/
│   ├── kroki-core/          # DiagramProvider, providers, config, plugins, browser abstraction
│   ├── kroki-server/        # Axum server, middleware, admin, metrics
│   └── kroki-cli/           # CLI binary
```

- [ ] **kroki-core crate**: Pure library with no server dependencies. Owns providers, config, plugins, browser abstraction.
- [ ] **kroki-server crate**: Axum server, middleware, admin dashboard. Depends on `kroki-core`.
- [ ] **kroki-cli crate**: CLI binary. Depends on `kroki-core`.

## 🌐 v0.1.0: The Unified Vision
- [ ] **Live Preview Web UI**: A management dashboard to test conversions and view server health.
- [ ] **Edge Deployment**: Optimize for Cloudflare Workers / Fastly Compute@Edge contexts.
- [ ] **CloudEvents**: Adopt CloudEvents for rendering triggers (CNCF alignment).

---
> Have a suggestion? Feel free to open an [Issue](https://github.com/softmentor/kroki-rs/issues) or a [Pull Request](https://github.com/softmentor/kroki-rs/pulls).
