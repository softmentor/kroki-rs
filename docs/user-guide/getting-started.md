---
title: Getting Started with Kroki-rs
label: kroki-rs.user-guide.getting-started
---
# Getting Started with Kroki-rs

## Prerequisites

Kroki-rs is highly modular and relies on external tools to render specific diagrams. The [**Installation Script**](#the-quick-way-recommended) will check for these automatically, but you should eventually install:

- **Graphviz**: For DOT and PlantUML support.
- **Node.js**: Required for Mermaid, Vega, and Wavedrom.
- **D2**: For modern, data-driven diagrams.

See the [**Supported Diagrams**](#kroki-rs.user-guide.supported-diagrams) page for details on tool requirements.

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

## Next Steps
- Explore [**Usage Details**](#kroki-rs.user-guide.usage) for advanced CLI flags.
- Check [**Supported Diagrams**](#kroki-rs.user-guide.supported-diagrams) to see what you can build.
