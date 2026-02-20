---
title: Security Guide
label: kroki-rs.user-guide.security
---
# Security Considerations for Public Hosted Instances

When hosting `kroki-rs` in the public domain, it is critical to implement standard infrastructure security practices. While `kroki-rs` uses secure temporary file handling, adaptive timeouts, and zombie process mitigation, it is ultimately executing external binaries that parse untrusted user input.

## 1. Do Not Expose Directly
Never expose the raw `kroki-rs` binary directly to the public internet. Always place it behind a reputable reverse proxy such as **Nginx**, **Caddy**, or **Cloudflare**.

- **TLS/SSL Encryption:** Enforce HTTPS to protect diagram payload transit.
- **Rate Limiting:** Implement strict API rate limiting to prevent automated scraping or massive concurrent generation attacks.

## 2. Network Isolation
Diagram generation tools rarely need outbound internet access (except for the built-in dynamic font downloader, which only fetches explicitly configured URLs).
- **Docker/Containerization:** Run the application inside an isolated Docker container.
- **Egress Filtering:** Block unnecessary outbound network connections at the firewall level to prevent Server-Side Request Forgery (SSRF) vulnerabilities in upstream parsers.

## 3. Resource Limits
Even with the internal adaptive timeout mechanisms natively protecting the Rust execution context, you must enforce strict hardware limits for the underlying machine.
- **Cgroups/Docker Limits:** Limit the maximum CPU quota and RAM availability for the container.
- **Process Limits (`ulimit`):** Restrict the maximum number of child processes the `kroki-rs` user can spawn.

## 4. Timeout Configurations
You can strictly control execution time by modifying the `kroki.toml` configuration file.
By default, the adaptive timeout will kill hung processes within ~10 seconds. You can clamp this down:
```toml
[default]
# Enforce a global hard limit across all providers
timeout_ms = 4000
```

## 5. Security Updates
Monitor the upstream repositories for the individual CLI tools (e.g., Mermaid CLI, D2, Graphviz) used by `kroki-rs` and ensure their binaries are kept up to date.
