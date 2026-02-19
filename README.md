# Kroki-rs

**Kroki-rs** is a lightweight, high-performance Rust port of the popular [Kroki](https://kroki.io) diagram generation service. It provides a unified API to convert text-based diagram descriptions (like Graphviz, Mermaid, PlantUML, ZenUML, etc.) into images (SVG, PNG, etc.).

Unlike the original Java-based Kroki which bundles dependencies in Docker, **Kroki-rs** is designed to run natively on your system, leveraging installed CLI tools and providing a blazingly fast and resource-efficient alternative.

## Features

-   🚀 **Fast & Lightweight**: Written in Rust, minimal footprint.
-   🛠️ **Native Execution**: Runs directly on host, using system tools or Node.js versions.
-   🔄 **Drop-in Compatible**: specific API endpoints match Kroki (`GET /:type/:format/:source`).
-   🖥️ **CLI Support**: Convert files instantly from the command line.
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

Start the server:

```bash
./target/release/kroki-rs serve
```

Generate a diagram via API:

```bash
curl http://localhost:8000/graphviz/svg/eNpLyUwvSizm5TIGAAWDAY0=
```

## Documentation

Full documentation is available in the [`docs/`](docs/) directory and served on GitHub Pages.

-   [Getting Started](docs/getting-started.md)
-   [Supported Diagrams](docs/supported-diagrams.md)
-   [Configuration](docs/configuration.md)
-   [Developer Guide](docs/developer-guide.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
