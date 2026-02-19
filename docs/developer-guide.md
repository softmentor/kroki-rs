# Developer Guide

This guide is for developers who want to contribute to **Kroki-rs** or understand its internal architecture.

## Architecture Overview

Kroki-rs is built as a modular system using the **Provider Pattern**. This decouples the HTTP API and CLI from the specific logic required to generate each diagram type.

### Project Structure

```
├── scripts/         # Maintenance and distribution scripts
├── tests/           # Integration tests and fixtures
├── src/
│   ├── main.rs          # Entry point, CLI parsing, capability discovery
│   ├── capabilities.rs  # Logic for discovering and logging available tools
│   ├── utils/           # Shared utility functions
│   │   └── mod.rs       # Payload decoding (Deflate/Base64)
│   ├── server/          # Axum web server
│   │   ├── mod.rs       # Server setup and routing
│   │   └── handlers.rs  # API route handlers
│   ├── cli/             # CLI logic
│   │   └── mod.rs       # One-shot conversion logic
│   ├── config/          # Configuration system
│   │   └── mod.rs       # kroki.toml loading and schema
│   └── diagrams/        # Diagram processing logic
│       ├── mod.rs       # DiagramProvider trait definition
│       ├── registry.rs  # Map of type names to provider implementations
│       └── providers/   # Individual provider implementations
│           ├── cmd.rs       # Generic command line wrapper
│           ├── mermaid.rs   # Specialized mermaid handler
│           ├── plantuml.rs  # Specialized plantuml handler
│           ├── vega.rs      # Vega and Vega-Lite handler
│           ├── wavedrom.rs  # Wavedrom handler
│           ├── bpmn.rs      # BPMN handler
│           ├── d2.rs        # D2 handler
│           └── ditaa.rs     # Ditaa handler
```

## Adding a New Provider

To add support for a new diagram type:

1.  **Define the Provider**: Create a new file in `src/diagrams/providers/your_thing.rs`.
    -   Implement the `DiagramProvider` trait.
    -   Handle input (stdin/file) and capture output (stdout/file).

2.  **Expose the Module**: Add `pub mod your_thing;` to `src/diagrams/providers/mod.rs` (if it exists) or `src/diagrams/mod.rs`.

3.  **Update Capabilities**: Add any new binaries to the `Capabilities` struct in `src/capabilities.rs` so they are discovered on startup.

4.  **Register the Provider**: Add an entry to the `DiagramRegistry::new` function in `src/diagrams/registry.rs`.

## Coding Standards

-   **Error Handling**: Use `anyhow` for flexible error reporting.
-   **Formatting**: Always run `cargo fmt` before committing.
-   **Linting**: Ensure `cargo clippy` is clean.

## Testing

Kroki-rs uses standard Rust testing practices:

### Unit & Integration Tests
Run the test suite using the Makefile:

```bash
make test
```

To see output from tests (e.g., skips due to missing tools):

```bash
make test-v
```

### Distribution Verification
To verify the build, packaging, and conversion process in one command:

```bash
make verify
```
This target runs lints, tests, builds the release binary, creates a tarball, and verifies the binary's functionality.

## Internal Workflow: Payload Decoding

Kroki-rs implements the standard Kroki encoding scheme:
1.  **Input**: Text description.
2.  **Compress**: Zlib (RFC 1950) or Deflate (RFC 1951).
3.  **Encode**: Base64URL.

The server handles decoding automatically in the `/[:type]/[:format]/[:source]` route.

## Debugging & Logging

Kroki-rs uses `tracing` for structured logging.

### Controlling Log Levels
Use the `RUST_LOG` environment variable to control output:

- **General Usage**: `RUST_LOG=info kroki-rs serve`
- **Tool Discovery Debugging**: `RUST_LOG=debug kroki-rs serve` (Shows paths to all discovered binaries).
- **External Command Failures**: Errors from underlying tools are logged at the `error` level.

### Makefile Helpers
The Makefile targets also respect `RUST_LOG`:
```bash
RUST_LOG=debug make test
```
