---
title: Local Development Guide
label: kroki-rs.developer-guide.local-dev
---

# Local Development Guide

This guide covers the practical tools and structure used for day-to-day development in Kroki-rs.

## Project Structure

```
├── dflow             # Professional developer flow orchestrator
├── src-scripts/      # Modular shell scripts (setup, develop, ci-verify, gh-tasks)
├── docs/             # MyST documentation (User & Developer guides)
├── src/
│   ├── main.rs       # Entry point and capability discovery
│   ├── server/       # Axum web server and API handlers
│   ├── cli/          # One-shot conversion logic
│   ├── config/       # kroki.toml schema and loading
│   └── diagrams/     # Diagram processing and Provider trait
│       ├── registry.rs  # Provider mapping
│       └── providers/   # Individual diagram engines (D2, Mermaid, etc.)
├── tests/            # Integration tests and fixtures
└── resources/        # Static assets (browser harness, libraries)
```

## The `dflow` CLI

`dflow` is our professional orchestrator for all development tasks.

| Command | Alias | Description |
| :--- | :--- | :--- |
| `./dflow setup` | - | Initialize local environment and base images. |
| `./dflow develop` | `dev` | Rapid native-OS iteration (Fast feedback). |
| `./dflow ci-verify`| `repro` | Containerized verification (CI Parity). |
| `./dflow ci-shell` | `shell` | Interactive shell inside the CI container. |
| `./dflow gh`       | - | Production CI (Remote trigger). |
| `./dflow teardown` | `clean` | Reclaim build and container disk space. |

## `dflow` Options Reference

All commands accept the following options to customize the behavior:

| Option | Long Form | Description |
| :--- | :--- | :--- |
| `-p` | `--purge` | Purge all caches (dist, target) before running. |
| `-d` | `--debug` | Enable `RUST_LOG=debug` for detailed tracing. |
| `-v` | `--verbose` | Enable verbose tool output (e.g., `cargo --verbose`). |
| `-n` | `--no-network` | Restricted build mode (no downloads). |
| `-t load` | `--test=load` | Include high-concurrency stress tests. |
| `-j <n>` | `--jobs=<n>` | Limit parallelism to `<n>` threads. |
| `--lean` | - | Build core only (skips browser engine). |

### Granular Execution
The `ci-verify` command supports passing sub-targets directly to the containerized environment:
```bash
# Only run format check in the container
./dflow ci-verify fmt

# Only run tests in the container
./dflow repro test-ci
```

## Resource Optimization

- **`sccache`**: Mandatory for fast local builds. Detectable by the Makefile.
- **Podman Memory**: Ensure at least **4GB** (6GB recommended) for `native-browser` builds.

## Adding a New Provider

1.  **Define**: Create `src/diagrams/providers/your_thing.rs`.
2.  **Implement**: Follow the [Coding Standards](#kroki-rs.developer-guide.coding-patterns).
3.  **Discovery**: Add to `Capabilities` in `src/capabilities.rs`.
4.  **Register**: Add to `DiagramRegistry::new` in `src/diagrams/registry.rs`.

## Testing

- **Native**: `make test` or `make verify`.
- **Containerized**: `bash src-scripts/ci-verify/repro-ci.sh`.
