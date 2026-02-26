---
title: "ADR 0004: Universal Browser Rendering Pool"
label: kroki-rs.adr.0004
---

# ADR 0004: Universal Browser Rendering Pool

| Field | Value |
| :--- | :--- |
| **Status** | Accepted (TTL Recycling: Planned) |
| **Updated** | 2026-02-26 |

## Context

Diagram providers like **Mermaid**, **BPMN**, and **Excalidraw** require a browser environment (DOM, Canvas, SVG engine) to render accurately. 

Initially, we planned to use **Playwright** (Node.js) with a persistent daemon and `generic-pool`. However, to achieve a **Zero-Dependency Core**, we migrated to a pure-Rust implementation using `headless_chrome`.

## Decision

We have implemented a **Native Browser Rendering Pool** directly in Rust.

1.  **Backend**: Uses the `headless_chrome` crate to communicate with Chromium via the Chrome DevTools Protocol (CDP).
2.  **Pooling Strategy**:
    -   A persistent `Browser` instance is launched at startup.
    -   Requests are handled via a pool of `BrowserContext` objects (Incognito-like isolation).
    -   **TTL Recycling (Planned)**: Each context will be recycled after a configurable number of requests to prevent cumulative memory bloat. *Currently tracked as technical debt (TD-22).*
3.  **Harnessing**: Diagrams are rendered by injecting minimized library bundles into the page context and capturing the resulting HTML/SVG.

## Consequences

-   **Positive**: Zero Node.js or Playwright runtime dependency.
-   **Positive**: Dramatically reduced container image size (no Node/NPM).
-   **Positive**: Lower latency due to elimination of inter-process communication with a Node.js daemon.
-   **Configuration**: Settings originally intended for Playwright (`pool_size`) are applied directly to the native Rust pool. `context_ttl_requests` is currently a placeholder in the config.
