# Getting Started with Kroki-rs

## Prerequisites

Before you begin, ensure you have the following installed:

1.  **Rust Toolchain**: Required to build the project.
    -   Install via [rustup.rs](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.  **Diagram Tools**: Kroki-rs relies on external tools to generate diagrams. Install the ones you need:
    -   **Graphviz**: `brew install graphviz` (macOS) or `apt-get install graphviz` (Linux).
    -   **Node.js**: Required for Mermaid, Vega, Wavedrom, etc.
        -   Run `npm install` in the project root to install Node-based tools locally in `node_modules`.

## Installation

### From Source

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

## Running the Application

### Setup Capabilities

On startup, `kroki-rs` automatically checks for available tools in your `PATH` and `node_modules/.bin`.

1.  Install Node dependencies (optional, for Mermaid/Vega/etc.):
    ```bash
    npm install
    ```

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

### Verify Installation

Check the health endpoint (if implemented) or try a simple conversion:

```bash
echo "digraph G { A -> B }" > test.dot
./target/release/kroki-rs convert -t dot -f svg test.dot
```
