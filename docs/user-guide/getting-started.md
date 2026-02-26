---
title: Getting Started with Kroki-rs
label: kroki-rs.user-guide.getting-started
---
# Getting Started with Kroki-rs

## Prerequisites

Kroki-rs is highly modular. While many diagrams now use a **Rust-Native** engine with zero runtime dependencies, some specialized tools still require external binaries:

- **Graphviz**: Required for Ditaa and advanced DOT features.
- **Node.js**: Required for Vega and Wavedrom (Mermaid and BPMN are now native).
- **D2**: For modern, data-driven diagrams.
- **Chromium / Chrome**: Required for browser-based diagrams (Mermaid, BPMN).


## Installation

(the-quick-way-recommended)=
### The Quick Way (Recommended)
Install `kroki-rs` and verify dependencies with a single command:

```bash
curl -sSfL https://raw.githubusercontent.com/softmentor/kroki-rs/main/install.sh | sh
```

### From Source
If you prefer to build from source:

1.  Clone the repository:
    ```bash
    git clone https://github.com/your-username/kroki-rs.git
    cd kroki-rs
    ```

2.  Build the release binary:
    ```bash
    cargo build --release
    ```

    The binary will be located at `./target/release/kroki-rs`.

### Via Docker
If you prefer not to manage external dependencies like Chromium, Node, or Graphviz yourself, you can use the official multi-arch Docker image:

```bash
docker run --rm -p 8000:8000 -p 8081:8081 ghcr.io/softmentor/kroki-rs:latest
```
This runs the main API on port `8000` and the Admin dashboard/health checks on port `8081`. 

For detailed production guidelines, including monitoring and configuration, see the [**Docker Deployment Guide**](#kroki-rs.user-guide.docker-deployment). For other installation methods, see [**Releases & Installation**](#kroki-rs.user-guide.releases).

## Running the Application

### Setup Capabilities

On startup, `kroki-rs` automatically checks for available tools in your `PATH` and `node_modules/.bin`.

1.  (Optional) Install Node dependencies:
    ```bash
    npm install
    ```
    *Note: Mermaid and BPMN libraries are already embedded in the binary. This is only needed for Vega or other Node-based CLI tools.*

2.  Run the server:
    ```bash
    ./target/release/kroki-rs serve
    ```

    You should see logs indicating which tools were discovered.

    ```text
    Debug: Capabilities discovery:
      Graphviz (Some("dot")): Some("/usr/bin/dot")
      Mermaid (Some("mmdc")): Some("node_modules/.bin/mmdc")
      ...
    ```

### Quick Start (Server)
Start the high-performance server in a single command:
```bash
kroki-rs serve --port 8000
```

### Quick Start (CLI)
Convert a diagram file directly without starting the server:
```bash
kroki-rs convert --type mermaid --format svg test.mmd
```

## Quick Example
Once the server is running, you can generate an SVG from a Graphviz dot file using `curl`:

```bash
curl http://localhost:8000/graphviz/svg/eNpLyUwvSizm5TIGAAWDAY0=
```


## Development & Contribution

If you are contributing to Kroki-rs or running the full verification suite, we recommend using the `dflow` orchestration tool.

### Development Prerequisites
- **Rust**: Latest stable toolchain via [rustup.rs](https://rustup.rs).
- **Make**: Standard build automation (GNU Make).
- **Podman / Docker**: Required for containerized CI verification (`./dflow ci-verify`).
- **GitHub CLI (gh)**: Required for proposing releases (`./dflow release -b`).

### Using `dflow`
The `dflow` script provides a unified interface for all development tasks:
- `./dflow setup`: Initializes the environment (Podman machines, toolchains).
- `./dflow dev`: Runs local native verification (fmt, lint, test, smoke-test).
- `./dflow ci-verify`: Runs the **exact** same pipeline as GitHub Actions using a pre-baked CI container.
- `./dflow teardown`: Reclaims disk space and prunes old caches.

## Next Steps
- Explore [**Usage Details**](#kroki-rs.user-guide.usage) for advanced CLI flags.
- Check [**Supported Diagrams**](#kroki-rs.user-guide.supported-diagrams) to see what you can build.
