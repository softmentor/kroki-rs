---
name: kroki_rs_integration
description: Integration guide for tools and LLMs to consume Kroki-rs for diagram generation.
---

# Kroki-rs Integration Skill

This guide helps AI agents and developers integrate **Kroki-rs** into their workflows.

## Overview
Kroki-rs converts text-based diagram descriptions (Mermaid, Graphviz, D2, etc.) into images (SVG, PNG, WebP). It is a drop-in replacement for the original Kroki service.

## Usage Modes

### 1. CLI Usage
Ideal for one-off conversions or build scripts.
```bash
kroki-rs convert --type <TYPE> --format <FORMAT> [--font <TTF_URL>] <INPUT_FILE>
```
- **Inputs**: File path or stdin.
- **Outputs**: Stdout (redirect to file).
- **Optional**: `--font` URL to dynamically download and embed a `.ttf` file.

### 2. Server (HTTP API)
Kroki-rs provides a REST API compatible with the [Kroki specification](https://docs.kroki.io/kroki/setup/http-api/).

**GET Endpoint**: `GET /:type/:format/:encoded_source`
- **Encoding Algorithm**:
    1.  Take the diagram text (UTF-8).
    2.  Compress using **Zlib** (RFC 1950) or **Deflate** (RFC 1951).
    3.  Encode using **Base64URL** (replace `+` with `-` and `/` with `_`).

#### Implementation Example (Python)
```python
import zlib
import base64

def encode_diagram(text):
    zlib_compressed = zlib.compress(text.encode('utf-8'))
    return base64.urlsafe_b64encode(zlib_compressed).decode('utf-8')

# Usage
type = "mermaid"
format = "svg"
source = "graph TD; A-->B"
url = f"http://localhost:8000/{type}/{format}/{encode_diagram(source)}"
```

#### Implementation Example (JavaScript)
```javascript
const pako = require('pako');

function encodeDiagram(text) {
  const data = Buffer.from(text, 'utf8');
  const compressed = pako.deflate(data);
  return Buffer.from(compressed)
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');
}
```

## Supported Diagram Types
Common types: `mermaid`, `dot` (Graphviz), `plantuml`, `d2`, `vega`, `vegalite`, `wavedrom`, `bpmn`, `ditaa`.
## Debugging & Logging
Kroki-rs uses the `tracing` ecosystem. You can control logging levels using the `RUST_LOG` environment variable.

Supported levels: `error`, `warn`, `info`, `debug`, `trace`.

### Examples
Start the server with info logging:
```bash
RUST_LOG=info kroki-rs serve
```

Debug tool discovery issues:
```bash
RUST_LOG=debug kroki-rs serve
```

Log everything except common dependencies:
```bash
RUST_LOG=kroki_rs=debug,warn kroki-rs serve
```

## Key Project Files
- **Binary**: `target/release/kroki-rs`
- **Configuration**: `kroki.toml` (overrides tool paths).
- **Makefile**: Use `make verify` to check project health or `make serve` to start the API.

## Error Handling
The API returns standard HTTP status codes:
- `400 Bad Request`: Encoding or decoding failure.
- `404 Not Found`: Unsupported diagram type or missing local tool.
- `500 Internal Server Error`: Tool execution failure.
