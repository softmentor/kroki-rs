# Kroki-rs

**Kroki-rs** is a lightweight, high-performance Rust port of the popular [Kroki](https://kroki.io) diagram generation service. It provides a unified API to convert text-based diagram descriptions (like Graphviz, Mermaid, PlantUML, ZenUML, etc.) into images (SVG, PNG, etc.).

Unlike the original Java-based Kroki which bundles dependencies in Docker, **Kroki-rs** is designed to run natively on your system, leveraging installed CLI tools and providing a blazingly fast and resource-efficient alternative.

## Features

-   🚀 **Fast & Lightweight**: Written in Rust, minimal footprint.
-   🛠️ **Native Execution**: Runs directly on host, using system tools or Node.js versions.
-   🏊‍♂️ **Browser Instance Pooling**: Employs a Node.js daemon with generic-pool to seamlessly recycle `Playwright` contexts for zero-latency Mermaid and BPMN rendering.
-   🔄 **Drop-in Compatible**: specific API endpoints match Kroki (`GET /:type/:format/:source`).
-   🖥️ **CLI Support**: Convert files instantly from the command line, with new **batch processing** capabilities.
-   ⚡ **Built-in Caching**: Optional filesystem cache using SHA-256 for instant re-rendering of unchanged diagrams.
-   🔍 **Auto-Discovery**: Automatically detects installed diagram tools.

## Quick Start

### Installation

Clone the repository and build with Cargo:

```bash
git clone https://github.com/your-username/kroki-rs
cd kroki-rs
cargo build --release
```

### Usage (CLI)

Convert a Graphviz dot file to SVG:

```bash
echo "digraph G { Hello -> World }" > hello.dot
./target/release/kroki-rs convert -t dot -f svg hello.dot > hello.svg
```

### Usage (Server)

Start the server natively:

```bash
./target/release/kroki-rs serve
```

### Usage (Docker)

You can run the fully self-contained OCI image, which bundles all necessary tools like Chromium and Node.js:

```bash
docker run --rm -p 8000:8000 -p 8081:8081 ghcr.io/softmentor/kroki-rs:latest
```

Generate a diagram via API:

```bash
curl http://localhost:8000/graphviz/svg/eNpLyUwvSizm5TIGAAWDAY0=
```

## Documentation

Full documentation is available in the [`docs/`](docs/) directory and served on GitHub Pages.

-   [Getting Started](docs/user-guide/getting-started.md)
-   [Supported Diagrams](docs/user-guide/supported-diagrams.md)
-   [Configuration](docs/user-guide/configuration.md)
-   [Developer Guide](docs/developer-guide/developer-index.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
