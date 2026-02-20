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
admin_port = 8081
timeout_ms = 5000

[browser]
pool_size = 4
context_ttl_requests = 100

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

Kroki-rs configuration follows a strict priority scale:
**CLI Arguments > Environment Variables > `kroki.toml` > Defaults**

You can override any configuration directly using environment variables:

-   `KROKI_PORT`: Server port (default 8000)
-   `KROKI_ADMIN_PORT`: Admin dashboard & health check port (default 8081)
-   `KROKI_TIMEOUT`: Global fallback timeout in milliseconds
-   `KROKI_MAX_INPUT_SIZE`: Maximum payload size in bytes
-   `KROKI_MAX_OUTPUT_SIZE`: Maximum SVG/PNG size in bytes
-   `KROKI_CONFIG`: Path to a custom config file
-   `KROKI_BROWSER_POOL_SIZE`: Maximum number of concurrent Playwright generic-pool browser sessions (default 4)
-   `KROKI_BROWSER_CONTEXT_TTL`: Request evaluations allowed per browser context before forced recycling (default 100)

Tool-specific env overrides follow the pattern `KROKI_<TOOL>_<OPTION>`, e.g.:
-   `KROKI_GRAPHVIZ_BIN`: Path to dot executable
-   `KROKI_MERMAID_TIMEOUT`: Timeout specifically for Mermaid routines

## Capability Discovery

At startup, the application scans for tools in the following order:
1.  **Configuration**: Paths defined in `kroki.toml`
2.  **Local Node Modules**: `node_modules/.bin/` (for JS tools)
3.  **System Path**: `PATH` environment variable

If a tool is not found, its corresponding diagram types will be disabled.
