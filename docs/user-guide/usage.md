---
title: Usage Guide
label: kroki-rs.user-guide.usage
---
# Usage Guide

Kroki-rs can be used in two modes: **CLI** (Command Line Interface) and **Server** (HTTP API).

## CLI Mode

Use the CLI for one-off conversions or batch processing scripts.

### Command Structure

```bash
kroki-rs convert --type <DIAGRAM_TYPE> --format <OUTPUT_FORMAT> <INPUT_FILE>
```

-   `-t, --type`: The diagram type (e.g., `dot`, `mermaid`, `d2`, `excalidraw`).
-   `-f, --format`: The output format (`svg`, `png`, `pdf`, `webp`). *Note: SVG provides the best vector fidelity, but WebP is supported natively by kroki-rs via high-quality lossless rasterization.*
-   `<INPUT_FILE>`: Path to the file containing the diagram description.
-   `--cache-dir <DIR>`: (Optional) Path to enable local SVG/WebP caching to skip redundant conversions.
-   `--font <URL>`: (Optional) HTTP URL to a `.ttf` or `.otf` font file. Kroki-rs will download, cache, and automatically load this font into the rendering context (especially useful for achieving perfect WebP output fidelity for web-based tools like Mermaid). You can specify multiple `--font` flags.

### Batch Conversion

Have a folder full of diagrams? Kroki-rs can convert them all concurrently:

```bash
kroki-rs batch -f webp --out-dir ./output ./my-diagrams
```
*Kroki-rs will automatically detect the diagram type from the file extension (e.g., `.mmd` -> `mermaid`, `.d2` -> `d2`).*

### Examples

```{dropdown} Click to view conversion examples for all supported types
::::{tab-set}

:::{tab-item} Graphviz
**Command:**
```bash
kroki-rs convert -t graphviz -f svg tests/fixtures/test.dot > output.svg
```
:::

:::{tab-item} Mermaid
**Command:**
```bash
kroki-rs convert -t mermaid -f svg tests/fixtures/test.mmd > output.svg
```
:::

:::{tab-item} PlantUML
**Command:**
```bash
kroki-rs convert -t plantuml -f svg tests/fixtures/test.puml > output.svg
```
:::

:::{tab-item} D2
**Command:**
```bash
kroki-rs convert -t d2 -f svg tests/fixtures/test.d2 > output.svg
```
:::

:::{tab-item} BPMN
**Command:**
```bash
kroki-rs convert -t bpmn -f svg tests/fixtures/test.bpmn > output.svg
```
:::

:::{tab-item} Wavedrom
**Command:**
```bash
kroki-rs convert -t wavedrom -f svg tests/fixtures/test.json5 > output.svg
```
:::

:::{tab-item} Ditaa
**Command:**
```bash
kroki-rs convert -t ditaa -f png tests/fixtures/test.ditaa > output.png
```
:::

:::{tab-item} Excalidraw
**Command:**
```bash
kroki-rs convert -t excalidraw -f svg tests/fixtures/test.excalidraw > output.svg
```
:::

:::{tab-item} Vega
**Command:**
```bash
kroki-rs convert -t vega -f svg tests/fixtures/test.vega > output.svg
```
:::

:::{tab-item} Vega-Lite
**Command:**
```bash
kroki-rs convert -t vegalite -f svg tests/fixtures/test.vl.json > output.svg
```
:::

::::
```

---

## Server Mode

The server CLI follows the [Kroki API specification](https://docs.kroki.io/kroki/setup/http-api/).

### Start Server

```bash
kroki-rs serve [--port <PORT>]
```
Default port is `8000`.

### API Endpoints

#### `GET /:type/:format/:source_encoded`

Generates a diagram from an encoded source string.

-   **:type**: Diagram type (e.g., `graphviz`, `mermaid`).
-   **:format**: Output format (`svg`, `png`, etc.).
-   **:source_encoded**: The diagram source code, compressed with **Zlib** (or Deflate) and encoded in **Base64URL**.

**Example:**
Request a Graphviz diagram:
```bash
curl http://localhost:8000/graphviz/svg/eNpLyUwvSizm5TIGAAWDAY0=
```

#### `POST /`

(If implemented) Accepts a JSON payload with diagram source and type.

```json
{
  "diagram_source": "digraph G { Hello -> World }",
  "diagram_type": "graphviz",
  "output_format": "svg"
}
```

### Supported Clients

Since `kroki-rs` is API-compatible, you can use existing Kroki clients! Just point them to your local instance.

-   **VS Code**: [Kroki extension](https://marketplace.visualstudio.com/items?itemName=yyi.kroki) (set URL to `http://localhost:8000`)
-   **Obsidian**: Kroki plugin
-   **Browser**: Kroki integrations
