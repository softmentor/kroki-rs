---
title: "ADR 0008: Rust-Native Browser Automation (Eliminating Node.js)"
label: kroki-rs.adr.0008
---

## Context
The current `BrowserManager` (ADR 0004) uses a Node.js child process running Playwright to render browser-dependent diagrams (Mermaid, BPMN, Excalidraw). While effective, this requires Node.js as a runtime dependency, complicating deployment and increasing container image size.

The initial goal, achieved in v0.0.5, was to eliminate the Node.js dependency entirely by replacing Playwright with a pure-Rust browser automation solution.

## Candidates

| Approach | Crate | Protocol | Node.js? | Notes |
|----------|-------|----------|----------|-------|
| **fantoccini** | [`fantoccini`](https://github.com/jonhoo/fantoccini) | W3C WebDriver | ❌ | Pure Rust, cross-browser, 75+ contributors. Requires `chromedriver`/`geckodriver` binary. |
| **headless_chrome** | [`headless_chrome`](https://crates.io/crates/headless_chrome) | Chrome DevTools Protocol | ❌ | Puppeteer-like API in Rust. Chrome-only. Active development (v1.0.20, Dec 2025). |
| **chromiumoxide** | [`chromiumoxide`](https://crates.io/crates/chromiumoxide) | Chrome DevTools Protocol | ❌ | Async CDP client. Chromium-only. Good CDP coverage. |
| **Current baseline** | Node.js + Playwright | CDP | ✅ | Mature, proven. Requires Node.js runtime. |

## Decision
**Run a standalone experiment first.** Do NOT modify the main codebase until results are captured.

### Experiment Design
1. Create a minimal Rust binary that:
   - Launches a headless Chrome/Chromium instance
   - Loads a page containing Mermaid.js
   - Evaluates a diagram source → captures SVG output
   - Converts SVG to PNG/WebP
2. Implement the above using each of the 3 candidates.
3. Measure and compare:
   - **Dependencies**: Number of transitive deps, required external binaries
   - **Binary size**: Delta vs. current kroki-rs binary
   - **Cold start latency**: Time from process start to first render
   - **Per-diagram latency**: Average and P99 for Mermaid SVG, BPMN SVG
   - **Memory**: RSS under 50 concurrent diagram requests
   - **Capabilities**: SVG, PNG, WebP output support
4. Record results in this ADR.

### Migration Strategy (post-experiment)
1. Create `RustBrowserManager` using the winning crate, with connection pooling.
2. Serve rendering JS (mermaid.min.js, bpmn-viewer.js) via data URIs or embedded HTTP server.
3. Use `client.execute()`/`tab.evaluate()` to render diagrams and capture output.
4. Feature-flag: `native-browser` — when enabled, use `RustBrowserManager`; when disabled, fall back to Node.js `BrowserManager`.
5. Extract `BrowserBackend` trait to abstract both implementations.

## Experiment Results
*To be filled after experiment completion.*

| Metric | fantoccini | headless_chrome | chromiumoxide | Node.js+Playwright |
|--------|-----------|-----------------|---------------|---------------------|
| Deps (transitive) | — | — | — | — |
| Binary size delta | — | — | — | baseline |
| Cold start (ms) | — | — | — | — |
| Mermaid SVG P50 (ms) | — | — | — | — |
| Mermaid SVG P99 (ms) | — | — | — | — |
| BPMN SVG P50 (ms) | — | — | — | — |
| Memory @ 50 conc. (MB) | — | — | — | — |

## Consequences
- **Positive**: Eliminating Node.js reduces container image by ~100MB+ and removes an entire runtime dependency.
- **Positive**: Pure-Rust browser control enables tighter integration (no HTTP hop to worker.js).
- **Positive**: Feature flag ensures zero risk — fallback to proven Playwright path.
- **Negative**: Requires `chromedriver` or Chrome binary as external dependency (unavoidable for browser-rendered diagrams).
- **Risk**: Some candidates may have incomplete API coverage for `page.evaluate()` with complex JS libraries.
