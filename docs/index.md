# Welcome to Kroki-rs

**Kroki-rs** is a unified API for all your diagramming needs, written in Rust. It serves as a drop-in replacement for the original Kroki service, designed for performance and ease of local deployment.

## Why Kroki-rs?

-   **Performance**: Built with Rust and Axum for high throughput.
-   **No Docker Required**: Leverages your system's installed tools, avoiding the overhead of massive Docker images.
-   **Flexibility**: Supports a wide range of diagram tools (Graphviz, Mermaid, PlantUML, D2, Vega, etc.).
-   **CLI & Server**: Use it as a command-line tool or a web service.

## Documentation

-   [**Getting Started**](getting-started.md): Installation and setup instructions.
-   [**Usage Guide**](usage.md): How to use the CLI and Server API.
-   [**Supported Diagrams**](supported-diagrams.md): List of supported diagram types and required tools.
-   [**Configuration**](configuration.md): Customizing the service with `kroki.toml`.
-   [**Distribution**](distribution.md): Options for installing and distributing the CLI.
-   [**Developer Guide**](developer-guide.md): Architecture internals and contributing.

## Quick Example

Generate an SVG from a Graphviz dot file:

```bash
curl http://localhost:8000/graphviz/svg/eNpLyUwvSizm5TIGAAWDAY0=
```

## License

MIT License. See [LICENSE](../LICENSE) for details.
