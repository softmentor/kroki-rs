---
title: Configuration Reference
label: kroki-rs.user-guide.configuration
---
# Configuration

Kroki-rs is designed to work out-of-the-box with auto-discovery, but you can customize it using a `kroki.toml` file in the working directory.

## `kroki.toml` Structure

```toml
[server]
port = 8000
timeout_ms = 5000

[graphviz]
bin_path = "/usr/local/bin/dot"

[mermaid]
bin_path = "./node_modules/.bin/mmdc"
# Optional specific config for mermaid
config_path = "mermaid-config.json"

[plantuml]
bin_path = "java"
# arguments = ["-jar", "plantuml.jar"] # (Future support)

[d2]
bin_path = "d2"
```

## Environment Variables

(Future implementation) settings may be overridden by environment variables:

-   `KROKI_PORT`: Server port
-   `KROKI_GRAPHVIZ_BIN`: Path to dot executable

## Capability Discovery

At startup, the application scans for tools in the following order:
1.  **Configuration**: Paths defined in `kroki.toml`
2.  **Local Node Modules**: `node_modules/.bin/` (for JS tools)
3.  **System Path**: `PATH` environment variable

If a tool is not found, its corresponding diagram types will be disabled.
