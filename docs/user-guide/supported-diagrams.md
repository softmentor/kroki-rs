---
title: Supported Diagram Types
label: kroki-rs.user-guide.supported-diagrams
---
# Supported Diagram Types

Kroki-rs supports a wide variety of diagram types by leveraging existing CLI tools and a high-performance **Native Browser Engine**.

| ID | Name | Primary Format | Backend |
| :--- | :--- | :--- | :--- |
| `graphviz` | [Graphviz](https://graphviz.org/) | SVG | `dot` (CLI) |
| **`mermaid`** | [Mermaid](https://mermaid.js.org/) | SVG | **Native Engine** (Headless Chromium) |
| **`bpmn`** | [BPMN](https://bpmn.io/) | SVG | **Native Engine** (Headless Chromium) |
| `d2` | [D2](https://d2lang.com/) | SVG | `d2` (CLI) |
| `ditaa` | [Ditaa](http://ditaa.sourceforge.net/) | PNG | `ditaa` (CLI) |
| `excalidraw` | [Excalidraw](https://excalidraw.com/) | SVG | `excalidraw-to-svg` (CLI) |
| `vega` | [Vega](https://vega.github.io/vega/) | SVG | `vg2svg` (CLI) |
| `vegalite` | [Vega-Lite](https://vega.github.io/vega-lite/) | SVG | `vl2vg` & `vg2svg` (CLI) |
| `wavedrom` | [WaveDrom](https://wavedrom.com/) | SVG | `wavedrom-cli` (CLI) |

## Native Browser Engine (Mermaid & BPMN)

Starting from **v0.0.8**, Mermaid and BPMN are rendered using a native headless Chromium instance. This ensures:
- **Accuracy**: Perfect rendering using official JS libraries.
- **Performance**: Managed tab pooling and concurrency control.
- **Serverless Architecture**: Uses a local `file://` based harness, eliminating the need for internal HTTP servers or loopback networking.
- **Zero-Dependency Core**: No local Node.js or Playwright installation required (all assets are embedded).

### High-Performance Font Support
Our Native Engine features a robust font-management system:
1.  **Google Fonts**: Supported out-of-the-box via internet access in headless mode.
2.  **Dynamic Injection**: You can inject custom CSS (e.g., `@font-face` or `@import`) into the rendering harness at runtime via the `window.krokiFontCss` configuration.
3.  **Consistency**: Automatically uses `--font-render-hinting=none` to ensure pixel-perfect SVG generation across different operating systems.

## Installation of Tools
Most tools can be installed via your system package manager.

### macOS (Homebrew)
```bash
brew install graphviz d2 ditaa
# For Mermaid/BPMN rendering, kroki-rs uses its native engine;
# however, an external browser is recommended for local development:
brew install --cask chromium
```

### Note on Format
While Kroki-rs supports SVG, PNG, and PDF, **SVG** is the recommended format for most web-based documentation as it provides the best clarity and scalability.
