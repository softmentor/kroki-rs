# Kroki-rs

**Kroki-rs** is a lightweight, high-performance Rust port of the popular [Kroki](https://kroki.io) diagram generation service. It provides a unified API to convert text-based diagram descriptions into images (SVG, PNG, WebP, etc.).

Unlike the original Java-based Kroki, **Kroki-rs** is designed to run natively and efficiently, leveraging existing CLI tools and a specialized **Native Browser Engine** for modern diagrams like Mermaid and BPMN.

## Highlights

-   **Native Browser Engine**: Pure-Rust `headless_chrome` for Mermaid and BPMN rendering (zero Node.js/Playwright runtime).
-   **Consolidated 3-Job pipeline**: Unified CI (Prep-Build-Verify) with backgrounded parallel checks for instant PR status feedback.
-   **Deterministic Parity**: Local verification via `./dflow ci-verify` uses the **exact** same fingerprinted image as GHA for 100% reproducibility.
-   **Automatic Packaging**: Production OCI images (`ghcr.io/softmentor/kroki-rs`) are built, multi-arch verified, and smoke-tested on every release.
-   **Integrated Docs**: MyST-based documentation infrastructure baked into the CI for reliable, offline-capable validation.

## Features

-   🚀 **Fast & Lightweight**: Minimal footprint with sub-50ms cold starts.
-   🛠️ **Native Execution**: Runs directly on your host using system tools.
-   ⚡ **Built-in Caching**: SHA-256 content-based caching for instant re-rendering.
-   🔍 **Auto-Discovery**: Automatically detects and leverages your installed tools.
-   🖥️ **CLI & Server**: Powerful CLI for batch processing and a production-ready Web API.

### Quick Start

### Prerequisites

Before running `./dflow setup`, ensure the following tools are available on your system:

-   **Rust Toolchain**: [rustup.rs](https://rustup.rs) (Required for native builds).
-   **Podman / Docker**: Required for containerized verification (`./dflow ci-verify`, `ci-shell`) and base image generation.
    -   *Mac Users*: Ensure the Podman machine is initialized and running (`podman machine start`).
-   **Native Renderers** (Optional for `dev`):
    -   **Chromium / Chrome**: Required for the Native Browser Engine (Mermaid/BPMN).
    -   **Graphviz / D2**: Recommended for local native rendering.

### Installation

Clone and initialize your environment:

```bash
git clone https://github.com/softmentor/kroki-rs
cd kroki-rs
./dflow setup
```

### Development & Verification

Our professional workflow follows the POSIX-standard CLI pattern: `./dflow <command> [options]`.

```bash
./dflow develop   # Local native iteration (macOS); alias: dev
./dflow ci-verify # Verify in CI container (same env as GHA); alias: repro
./dflow ci-shell  # Interactive shell in CI container for debugging; alias: shell
./dflow release   # Release management (propose branch or tag); see --help
./dflow teardown  # Reclaim build/container disk space; alias: clean
```

**Concrete Examples:**
```bash
./dflow develop -p -v            # Local iteration with full purge (-p) and verbose (-v)
./dflow ci-verify --test load    # Containerized CI verification with load tests
./dflow ci-verify fmt            # Run only format check in container
./dflow release -b               # Propose a release branch (create PR)
./dflow release --tag            # Tag and push release (triggers distribution)
./dflow teardown -f -y           # Deep cleanup (GHA caches + Podman storage, no prompt)
```

For advanced users, the underlying `Makefile` targets and `make help` still provide full variable-based control.

### Usage (CLI)

Convert a Graphviz dot file to SVG:

```bash
echo "digraph G { Hello -> World }" > hello.dot
./target/release/kroki-rs convert -t dot -f svg hello.dot > hello.svg
```

### Usage (Server)

Start the server:

```bash
make serve
```

## Documentation

Full documentation is available in the [`docs/`](docs/) directory:

-   [PIPELINE & CI/CD](docs/developer-guide/04-infrastructure/automation.md) - **Recommended for contributors**
-   [Supported Diagrams](docs/user-guide/supported-diagrams.md)
-   [User Guide](docs/user-guide/user-index.md)
-   [Developer Guide](docs/developer-guide/developer-index.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
