---
title: Diagram Providers & Dependencies
label: kroki-rs.developer-guide.providers
---

# Diagram Providers & Dependencies

Kroki-rs uses the **Provider Pattern** to decouple the API from rendering logic.

## The Provider Pattern

All providers implement the `DiagramProvider` trait. This ensures a consistent interface for validation and generation.

```rust
#[async_trait]
impl DiagramProvider for MyProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> { ... }
    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> { ... }
}
```

## System Dependencies

Kroki-rs delegates rendering to specialized external tools. Here is the dependency map:

| Provider | Runtime | External Dependency | Status |
| :--- | :--- | :--- | :--- |
| **Graphviz** | Native (C) | `graphviz` (dot) | Core |
| **D2** | Native (Go) | `d2` | Core |
| **Ditaa** | **JRE (Java)** | `ditaa` | **Legacy Support** |
| **Mermaid** | Browser (JS) | `chromium` | Native Browser |
| **BPMN** | Browser (JS) | `chromium` | Native Browser |
| **Vega** | Browser (JS) | `chromium` | Native Browser |

> [!IMPORTANT]
> The JRE is maintained specifically to support **Ditaa**. If a completely Java-free footprint is required, Ditaa support would need to be disabled.

## Adding a New Provider

1.  Create `src/diagrams/providers/new_tool.rs`.
2.  Use `define_provider!(NewToolProvider)`.
3.  Register in `registry.rs` and update `capabilities.rs`.

See [Coding Standards](#kroki-rs.developer-guide.coding-patterns) for implementation details.
