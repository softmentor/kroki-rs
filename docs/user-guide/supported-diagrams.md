---
title: Supported Diagram Types
label: kroki-rs.user-guide.supported-diagrams
---
# Supported Diagram Types

Kroki-rs supports a wide variety of diagram types by leveraging existing CLI tools. Below is a list of supported types and their primary delivery format.

| ID | Name | Primary Format | Required Tool |
| :--- | :--- | :--- | :--- |
| `graphviz` | [Graphviz](https://graphviz.org/) | SVG | `dot` |
| `mermaid` | [Mermaid](https://mermaid.js.org/) | SVG | **Native Engine** (Chrome/Chromium) |
| `plantuml` | [PlantUML](https://plantuml.com/) | SVG | **Native Engine** (Chrome/Chromium) |
| `vega` | [Vega](https://vega.github.io/vega/) | SVG | `vg2svg` (Node.js) |
| `vegalite` | [Vega-Lite](https://vega.github.io/vega-lite/) | SVG | `vl2vg` & `vg2svg` (Node.js) |
| `wavedrom` | [WaveDrom](https://wavedrom.com/) | SVG | `wavedrom-cli` (Node.js) |
| `bpmn` | [BPMN](https://bpmn.io/) | SVG | **Native Engine** (Chrome/Chromium) |
| `d2` | [D2](https://d2lang.com/) | SVG | `d2` |
| `ditaa` | [Ditaa](http://ditaa.sourceforge.net/) | PNG | `ditaa` |
| `excalidraw` | [Excalidraw](https://excalidraw.com/) | SVG | `excalidraw-to-svg` (Node.js) |

## Installation of Tools

Most tools can be installed via your system package manager or `npm`.

### macOS (Homebrew)
```bash
brew install graphviz plantuml d2 ditaa
```

### Node.js based (Local to project)
Install these via the provided `package.json`:
```bash
npm install
```
This installs `mmdc`, `vg2svg`, `vl2vg`, `wavedrom-cli`, and `bpmn-to-image` into `node_modules/.bin`.

## Formats Note

While Kroki-rs aims to support all formats (SVG, PNG, PDF, etc.) where possible, **SVG** is the most reliable and highly recommended format across all providers. Some providers (like `ditaa`) may only support PNG.

## Skipped / Upcoming
- **BlockDiag**: Coming soon.
- **C4 with PlantUML**: Supported via the `plantuml` provider.
