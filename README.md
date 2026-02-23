# Kroki-rs

**Kroki-rs** is a lightweight, high-performance Rust port of the popular [Kroki](https://kroki.io) diagram generation service. It provides a unified API to convert text-based diagram descriptions into images (SVG, PNG, WebP, etc.).

Unlike the original Java-based Kroki, **Kroki-rs** is designed to run natively and efficiently, leveraging existing CLI tools and a specialized **Native Browser Engine** for modern diagrams like Mermaid and BPMN.

## v0.0.5 Stabilization Achievements

-   🚀 **Native Browser Engine**: Replaced heavy Node.js/Playwright dependencies with a pure-Rust `headless_chrome` implementation for Mermaid and BPMN. (Now with **Serverless Harness** for zero-networking overhead and maximum reliability).
-   🎨 **Robust Font Support**: Dynamic font injection and pixel-perfect rendering using `resvg`.
-   🏢 **Professional Workflow**: Verify in the same CI environment as GitHub Actions (Docker base + deps). Use `./dflow ci-verify` locally and `remote-ci.sh` for remote offload; both run in the CI container for reproducibility.
-   🔋 **Remote CI**: Secure SSH-based offloading via `remote-ci.sh` — runs ci-verify (container) on the remote host, not native builds.

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
./dflow ci-verify # Verify in CI container (same env as GHA); alias: repro — use before pushing
./dflow ci-shell  # Interactive shell in CI container for incremental test fixes; alias: shell
./dflow teardown  # Reclaim build/container disk space; alias: clean
```

For remote offload (same container-based verification on another host), use `REMOTE_HOST=user@host bash src-scripts/ci-verify/remote-ci.sh`.

**Concrete Examples:**
```bash
./dflow develop -p -v            # Local iteration with full purge (-p) and verbose (-v)
./dflow ci-verify --test load    # Containerized CI verification with load tests
REMOTE_HOST=user@remote bash src-scripts/ci-verify/remote-ci.sh   # Remote offload (container)
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

-   [PIPELINE & CI/CD](docs/developer-guide/pipeline.md) - **Recommended for contributors**
-   [Supported Diagrams](docs/user-guide/supported-diagrams.md)
-   [User Guide](docs/user-guide/user-index.md)
-   [Developer Guide](docs/developer-guide/developer-index.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
