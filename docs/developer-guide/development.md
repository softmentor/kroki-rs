---
title: How to build and contribute
label: kroki-rs.developer-guide.build-contribute
---
# Build & Contribute

This page provides a deep dive into the internal design of **Kroki-rs**, its project structure, and guidelines for adding new diagram providers.

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
    -   Use the `crate::diagrams::define_provider!(YourThingProvider);` macro to generate the boilerplate struct and `new` method.
    -   Implement the `#[async_trait] DiagramProvider` trait.
    -   Use the `crate::diagrams::run_process_with_timeout` helper to safely execute the external binary. This automatically protects against infinite loops and ReDoS using adaptive timeouts based on payload size.

2.  **Expose the Module**: Add `pub mod your_thing;` to `src/diagrams/providers/mod.rs` (if it exists) or `src/diagrams/mod.rs`.

3.  **Update Capabilities**: Add any new binaries to the `Capabilities` struct in `src/capabilities.rs` so they are discovered on startup.

4.  **Register the Provider**: Add an entry to the `DiagramRegistry::new` function in `src/diagrams/registry.rs`.

## Coding Standards

-   **Error Handling**: Use `anyhow` for flexible error reporting.
-   **Formatting**: Always run `cargo fmt` before committing.
-   **Linting**: Ensure `cargo clippy` is clean.

## Local Build Optimization

Adding large dependencies like image encoders (`usvg`, `resvg`, `image`) increases the baseline compilation time of Kroki-rs. We strongly encourage developers to install `sccache` to accelerate their local builds:

```bash
cargo install sccache
# Or on macOS: brew install sccache
```

The repository's `Makefile` is configured to automatically detect if `sccache` is available in your `PATH` and set `RUST_LOG=debug`. Subsequent `make all` iterations will then aggressively cache and skip recompilation of unchanged crates.

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

### Container Testing
For instructions on building and testing within OCI containers, see the **[Docker Developer Guide](#kroki-rs.developer-guide.docker)**.


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

## Documentation Standards

Kroki-rs uses MyST Markdown for documentation. To maintain a robust and path-independent documentation structure, all contributors must follow these standards:

### 1. Mandatory Frontmatter
Every `.md` file in the `docs/` directory (and root documentation like `ROADMAP.md`) must include a frontmatter block at the very top:

```yaml
---
title: Descriptive Page Title
label: kroki-rs.[category].[page-name]
---
```

### 2. Label Naming Convention
Labels are the "Source of Truth" for linking. They must follow a hierarchical dot-notation:
- **Core Reference**: `kroki-rs.glossary`, `kroki-rs.reference`
- **User Guide**: `kroki-rs.user-guide.[page-name]`
- **Developer Guide**: `kroki-rs.developer-guide.[page-name]`
- **ADRs**: `kroki-rs.adr.[number]`

### 3. Path-Independent Linking
**Never** use hard-coded file paths (e.g., `usage.md` or `../user-guide/usage.md`) for internal links. Always use the MyST label.

**Why are semantic labels better?**
- **Refactor Resilience**: If a file is moved across the directory structure, all links using its label remain valid. You only need to update the file's location in the Table of Contents (`toc.yml`).
- **Rename Safety**: Labels remain constant even if the underlying filename changes (as we did with `development.md` and `user-index.md`).
- **Global Scope**: Labels provide a unique, project-wide identifier, eliminating the cognitive load of calculating relative paths (`../../`) between deeply nested folders.
- **Improved Tooling**: Documentation parsers can validate label existence at build time, preventing "404 Not Found" errors far more reliably than static path strings.

- **Incorrect**: `[Usage](usage.md)` or `[Usage](../user-guide/usage.md)`
- **Correct**: `[Usage](#kroki-rs.user-guide.usage)`

This ensures that moving or renaming a file does not break incoming links across the documentation set.
