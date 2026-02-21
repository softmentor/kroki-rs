# ADR 0007: Custom Plugin API via Subprocess Protocol

## Status
Proposed

## Context
Users need to extend Kroki-rs with custom diagram providers without modifying the core codebase. The plugin system must work identically for both CLI and server usage, meaning it belongs in the core library (not the server layer).

## Decision

### Subprocess Protocol
Plugins are external executables that follow a simple stdin/stdout contract:
1. Kroki-rs pipes diagram source text to the plugin's **stdin**.
2. The plugin writes rendered output (SVG/PNG bytes) to **stdout**.
3. Exit code 0 = success; non-zero = failure (stderr captured for error messages).

This is the same pattern used by the built-in `CommandProvider` and `run_process_with_timeout()`.

### Configuration
```toml
[[plugins]]
name = "my-custom-diagram"
command = "/usr/local/bin/my-renderer"
args = ["--format", "{format}"]
stdin = true
formats = ["svg", "png"]
timeout_ms = 5000
```

`{format}` in `args` is template-substituted at invocation time with the requested output format.

### Registration
- Plugin commands are validated at startup via `which` (same as built-in capabilities).
- Missing commands are logged as warnings but do not prevent server startup.
- Plugins are registered in `DiagramRegistry` alongside built-in providers. Name collisions with built-in providers are rejected with an error.

### Core Library Placement
The `PluginProvider` struct and registration logic live in `src/diagrams/plugin.rs`, ensuring they are available to both CLI (`kroki-rs convert --type my-custom-diagram`) and server (`GET /my-custom-diagram/svg/...`) modes. This prepares for the v0.0.6 workspace split where plugins will be part of `kroki-core`.

## Consequences
- **Positive**: Any language can implement a plugin — just read stdin, write stdout.
- **Positive**: Existing `run_process_with_timeout()` provides timeout/error handling for free.
- **Positive**: Zero coupling between plugins and Kroki-rs internals.
- **Negative**: Subprocess overhead per invocation (~5-20ms). Acceptable for most use cases; high-throughput plugins may want the future in-process WASM extension point.

## TODO
- [ ] Security hardening for plugins, sandbox, etc.
- [ ] WASM extension point for high-throughput plugins
- [ ] Plugin registry UI for server mode
- [ ] Plugin discovery mechanism
- [ ] Plugin versioning, caching, metrics, error handling, retry, etc.