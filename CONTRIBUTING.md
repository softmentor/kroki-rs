# Contributing to Kroki-rs

Thank you for your interest in contributing to **Kroki-rs**! We welcome contributions from everyone.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/your-username/kroki-rs.git
    cd kroki-rs
    ```
3.  **Create a new branch** for your feature or fix:
    ```bash
    git checkout -b feature/my-awesome-feature
    ```

## Development Workflow

### Prerequisites

-   Rust (stable)
-   Dependencies for diagrams you want to test (e.g., Graphviz, Node.js tools)
-   **(Optional but Recommended)**: `sccache` for accelerating local builds.
    ```bash
    cargo install sccache
    ```
    *The Makefile will automatically detect and use it to drastically reduce recompilation times for large dependencies like image/resvg.*

### Building

```bash
cargo build
```

### Testing

Run the test suite:

```bash
cargo test
```

To test specific diagram conversions, you can use the CLI:

```bash
cargo run -- convert -t dot -f svg test.dot
```

## Pull Requests

1.  Ensure your code builds and passes tests.
2.  Format your code using `cargo fmt`.
3.  Run lints using `cargo clippy`.
4.  Submit a Pull Request against the `main` branch.
5.  Provide a clear description of your changes.

## Adding a New Diagram Provider

1.  Create a new module in `src/diagrams/providers/`.
2.  Implement the `DiagramProvider` trait.
3.  Register the provider in `src/diagrams/registry.rs`.
4.  Add a test case/verification step.
5.  Update the documentation.

## Code of Conduct

Please be respectful and kind to others. Harassment or abusive behavior will not be tolerated.
