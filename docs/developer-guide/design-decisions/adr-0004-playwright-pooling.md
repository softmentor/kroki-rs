---
title: "ADR 0004: Browser Instance Pooling & Recycling"
label: kroki-rs.adr.0004
---

## Context
Currently, Kroki-rs relies on external CLI tools like `mermaid-cli` and `bpmn-to-image` to generate certain diagrams. These wrappers internally launch a separate Node.js process and a complete headless Chromium browser instance for every single diagram conversion request.

This architecture has severe limitations:
1. **High Latency**: Launching Chromium takes 1-3 seconds, imposing a hard floor on conversion speed.
2. **Resource Exhaustion**: Concurrent requests spawn unbounded Chrome instances, leading to server CPU/Memory exhaustion and potential Out-Of-Memory (OOM) crashes.
3. **Redundant Dependencies**: We manage multiple discrete Puppeteer wrappers, each downloading huge browser binaries.

## Decision
We will eliminate per-request browser launching by creating a **Persistent Browser Worker Daemon** using **Playwright**.

1. **Daemon Architecture**: Kroki-rs will launch a single, persistent Node.js background process at startup that maintains a warm Playwright `Browser`.
2. **Standardized Pooling Library**: We will use the `generic-pool` npm library to manage a pool of `BrowserContext`s. `generic-pool` is a battle-tested industry standard that natively handles:
    - Minimum and maximum pool sizes (e.g., keeping 2-10 contexts warm).
    - Eviction of idle resources.
    - Queuing when the pool is fully utilized.
3. **Strict TTL/Usage Limits**: To prevent inevitable Chromium memory leaks over long uptimes, the pool will destroy and recycle a `BrowserContext` after a predefined number of evaluations (e.g., 100 requests) or if the context encounters a severe crash.
4. **Universal Evaluation**: Instead of calling discrete CLI binaries, the Node.js daemon will use `page.evaluate(...)` to directly execute the client-side JavaScript libraries (e.g., `mermaid.min.js`, `bpmn-viewer.js`) within the pooled contexts.

### Why generic-pool?
While tools like `puppeteer-cluster` exist, they tightly couple the queueing logic with job runners and are primarily Puppeteer-focused. `generic-pool` is an abstract resource manager, allowing us to build a thin, explicit wrapper around Playwright `BrowserContext`s right inside an Express/Fastify HTTP server.

## Detailed Design Considerations

1. **DOM Footprint Recycling**: A critical discovery during load testing was that repeated evaluations within the same `page` context pollute the Document Object Model. Libraries like Mermaid attempt to initialize anchors (e.g., `#graphDiv`) that only exist on the first render. To safely recycle `BrowserContext`s across hundreds of usages, the daemon explicitly resets `document.body.innerHTML` before each generation, ensuring a clean slate.
2. **Crash Resilience and Teardown**: If evaluation throws an unrecoverable exception or crashes the underlying browser tab, `worker.js` instructs `generic-pool` to `destroy()` the bad context rather than `release()` it, triggering an automatic scale-up of a fresh Chromium `Page` proxy.
3. **Synchronous Node.js Blockers**: Playwright operations (`page.evaluate`) are highly asynchronous over CDP (Chrome DevTools Protocol). This means `worker.js` effortlessly juggles scores of simultaneous diagram rendering requests in parallel without blocking the Node Event Loop.

## Consequences
- **Positive**: Diagram rendering latency will plummet from ~2000ms to ~150ms.
- **Positive**: Memory usage becomes bounded and predictable, enabling high-throughput enterprise deployments.
- **Positive**: Unified dependency. We only need `playwright` directly, shedding layers of outdated wrapper code.
- **Negative**: Increased architectural complexity. Kroki-rs must now monitor and manage the lifecycle of a child Node.js HTTP server.
