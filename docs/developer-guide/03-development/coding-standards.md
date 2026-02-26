---
title: Coding Patterns
label: kroki-rs.developer-guide.coding-patterns
---

# Coding Patterns

This page documents established coding patterns in Kroki-rs. Follow these when contributing code.

## Process Execution

All external tool invocations go through `run_process_with_timeout()`:

```rust
let output = crate::diagrams::run_process_with_timeout(
    "tool-name",            // Human-readable tool name for error messages
    cmd,                    // tokio::process::Command
    Some(source.as_bytes()),// Optional stdin input
    self.timeout_ms,        // Per-tool timeout from config
    source.len(),           // Input size for adaptive timeout
).await?;
```

**Rules:**
- Always pass the tool's human-readable name (e.g., `"mmdc"`, `"dot"`)
- Never call `cmd.spawn()` directly — the wrapper handles kill-on-drop, timeouts, and contextual errors
- Timeouts are adaptive: base 3s + 1s per 10KB, capped at configured max

## Provider Pattern

All diagram providers implement `DiagramProvider`:

```rust
#[async_trait]
impl DiagramProvider for MyProvider {
    fn validate(&self, source: &str) -> Result<()> {
        if source.trim().is_empty() {
            return Err(anyhow::anyhow!("Diagram source is empty"));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // ... build command and run
    }
}
```

### Browser-Based Providers

If a diagram tool requires a JavaScript runtime (e.g., Mermaid, BPMN), use the `BrowserManager`:

```rust
pub struct MyBrowserProvider {
    browser: Arc<BrowserManager>,
}

#[async_trait]
impl DiagramProvider for MyBrowserProvider {
    async fn generate(&self, source: &str, format: &str) -> Result<Vec<u8>> {
        // The manager handles pooling, fallback, and native execution
        self.browser.evaluate("my-type", source, format).await
    }
}
```

**Rules:**
- Never initialize a new browser instance inside a provider — always use the shared `BrowserManager`
- Direct evaluation is preferred over subprocesses for performance
- The `type` passed to `evaluate()` must be supported by the browser harness (`window.kroki`)

**Rules:**
- Always validate format — **never** silently fall back to a different format
- Use `tokio::fs` (not `std::fs`) for file I/O inside async functions
- Use `define_provider!` macro for structural boilerplate
- Report tool-specific errors with the tool name

## Configuration Access

Use `Config` helper methods instead of inline logic:

```rust
// ✅ Good — uses centralized helper
let fonts = config.all_fonts();
let cache = Config::resolve_cache_dir(cli_override);

// ❌ Bad — duplicated inline logic
let mut fonts = Vec::new();
fonts.extend_from_slice(&config.mermaid.fonts);
fonts.extend_from_slice(&config.graphviz.fonts);
// ...
```

## Server State

The server uses `AppState` to inject shared state:

```rust
pub struct AppState {
    pub config: Config,
    pub registry: Arc<DiagramRegistry>,
}
```

**Rules:**
- `DiagramRegistry` is built **once** at startup — never per-request
- Capabilities are discovered once in `server::run()`
- Handler errors are logged internally via `tracing::error!()` and sanitized before returning to clients

## Input/Output Validation

All entry points (CLI and server) enforce:

| Check | Config Field | Default |
|-------|-------------|---------|
| Input size | `server.max_input_size` | 1MB |
| Output size | `server.max_output_size` | 50MB |
| Format whitelist | `SUPPORTED_FORMATS` | svg, png, pdf, webp, txt |

## Error Messages

Error messages should be actionable and identify the component:

```rust
// ✅ Good
anyhow::bail!(
    "'mmdc' timed out after {}ms (input: {} bytes). Consider increasing the timeout in kroki.toml.",
    timeout, size
);

// ❌ Bad
anyhow::bail!("Process timed out after {}ms", timeout);
```

## Async I/O

Inside `async fn`:
- Use `tokio::fs::read()` / `tokio::fs::write()` — **never** `std::fs`
- Use `tokio::process::Command` — **never** `std::process::Command`
- Blocking I/O on the async runtime starves other tasks

## Logging

- Use `tracing::info!()` for startup events, capability discovery
- Use `tracing::warn!()` for recoverable issues (cache miss, format fallback)
- Use `tracing::error!()` for generation failures
- Use `tracing::debug!()` for internal diagnostics
- **Never** use `println!()` in library or server code
