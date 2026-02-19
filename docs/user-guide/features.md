# Features

Kroki-rs is packed with features designed for speed, reliability, and ease of use.

## Unified Diagram API
- **Single Endpoint**: Access 10+ diagramming tools through a consistent HTTP interface.
- **CLI & Server**: Switch between one-shot CLI conversions and a persistent web service.
- **Drop-in Support**: Fully compatible with the original Kroki API specification for GET requests.

## Performance & Efficiency
- **Native Execution**: Runs CLI tools directly on your host, avoiding Docker overhead.
- **Rust Core**: Built with Axum and Tokio for high concurrency and minimal resource usage.
- **Lazy Discovery**: Tools are only discovered and initialized when needed.

## Developer Experience
- **Structured Logging**: Fine-grained control over log levels via `RUST_LOG`.
- **Comprehensive Docs**: Integrated Rustdoc and MyST documentation.
- **Robust Tooling**: A unified `Makefile` for the entire project lifecycle.
- **CI/CD Ready**: Automated GitHub Actions for releases and deployment.

## Supported Formats
- **Mermaid**: Flowcharts, Sequences, Gantt, etc.
- **Graphviz**: DOT language visualization.
- **D2**: Modern, declarative diagramming.
- **PlantUML**: Classic software modeling.
- **Vega/Vega-Lite**: Data visualization.
- **And more**: BPMN, WaveDrom, Ditaa.
