---
title: Browser Rendering Architecture
label: kroki-rs.developer-guide.browser-rendering
---

# Browser Rendering Architecture

Kroki-rs implements a **native Rust browser engine** strategy to eliminate the dependency on Node.js and external Puppeteer/Playwright CLI wrappers.

## Native Browser Strategy

We use the `headless_chrome` crate to communicate directly with Chromium via the DevTools Protocol (CDP).

- **Performance**: Zero-latency evaluation via pre-warmed tabs.
- **Security**: Strict process isolation and memory limits.
- **Simplicity**: No Node.js or `npm` required in the runtime environment.

## The Browser Manager

The `BrowserManager` orchestrates the lifecycle of the Chromium instance:

1.  **Tab Pooling**: A pool of pre-warmed tabs is maintained to handle bursts of diagram requests.
2.  **Harness Evaluation**: Diagrams are rendered using a specialized `browser-harness.js` that loads the required JS libraries (Mermaid, BPMN, etc.) from local resources.
3.  **Adaptive Recycling**: Tabs are gracefully recycled after a fixed number of requests to prevent memory accumulation.

## Font Management

Consistent font rendering is achieved by:
- Packaging core fonts (Inter, Roboto) in the Docker image.
- Mounting system font directories into the container.
- Configuring `fontconfig` to prioritize these paths.

## Supported Engines
- **Mermaid**: [Reference](#kroki-rs.developer-guide.providers)
- **BPMN**: [Reference](#kroki-rs.developer-guide.providers)
- **Vega / Vega-Lite**: [Reference](#kroki-rs.developer-guide.providers)
