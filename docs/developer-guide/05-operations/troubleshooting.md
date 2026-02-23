---
title: Maintenance & Troubleshooting
label: kroki-rs.developer-guide.troubleshooting
---

# Maintenance & Troubleshooting

This guide provides practical solutions for common issues encountered during development and operation.

## Common Issues & Solutions

### 1. Browser Worker Failures
- **Symptom**: Mermaid/BPMN renders return "Internal Server Error" or time out.
- **Check**: Run with `RUST_LOG=debug` to verify if Chromium is launching correctly.
- **Fix**: Ensure `CHROME_BIN` is pointing to a valid executable and your system has sufficient RAM (min 2GB).

### 2. Podman Resource Exhaustion
- **Symptom**: `./dflow ci-verify` hangs or the container crashes.
- **Fix**: Increase Podman VM memory:
  ```bash
  podman machine stop
  podman machine set --memory 4096
  podman machine start
  ```

### 3. Stale Caches
- **Symptom**: Changes in `Dockerfile` or `install.sh` aren't reflected in the CI run.
- **Fix**: Use the `--purge` flag:
  ```bash
  ./dflow ci-verify --purge
  ```

## Debugging Tools

### Interactive Container Shell
If a test fails only in the container, use `ci-shell` to debug in real-time:
```bash
./dflow ci-shell
# Inside: cargo test --test integration_tests
```

### Trace Profiling
For performance bottlenecks, use `tokio-console` (requires compilation with special flags) to inspect task scheduling.
