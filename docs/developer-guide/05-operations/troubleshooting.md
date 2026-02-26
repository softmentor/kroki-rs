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

### 2. Podman Resource Exhaustion & OOM
- **Symptom**: Compilation fails with `Signal 9` or the container crashes during heavy builds (e.g., `headless_chrome`).
- **Fix**: Increase Podman VM resources to at least 12GB RAM:
  ```bash
  podman machine stop
  podman machine set --memory 12288 --cpus 5
  podman machine start
  ```

### 3. Stale Caches & Fingerprint Mismatch
- **Symptom**: Changes in `Dockerfile` aren't reflected, or images are pulled unnecessarily.
- **Check**: Run `make print-base-fingerprint` and verify against `podman images`.
- **Fix**: Use `--purge` to force a clean slate:
  ```bash
  ./dflow ci-verify --purge
  ```

### 4. Exec Format Errors
- **Symptom**: `exec format error` when running the application inside the container.
- **Cause**: Reusing a `target/` directory compiled for the host (e.g., macOS binary on Linux).
- **Fix**: Ensure container builds use `target/ci` (automatically handled by `repro-ci.sh`). If stuck, wipe the target: `rm -rf target/ci`.

### 5. Sccache Connectivity Failures
- **Symptom**: `sccache` returns 400 Bad Request, fails to connect to the GHA backend, or `Operation not permitted` when running locally.
- **Fix**: Our infrastructure now uses **disk-based sccache**. Ensure `SCCACHE_GHA_ENABLED` is `false` and that `.cargo-cache/sccache` is correctly mounted and has write permissions. On systems where `sccache` cannot run (e.g., sandboxed macOS), disable the wrapper for the local session with `SCCACHE_ENABLED=false ./dflow dev` (or `./dflow ci-verify`). This skips the `RUSTC_WRAPPER` entirely without affecting CI.

## Debugging Tools

### Interactive Container Shell
If a test fails only in the container, use `ci-shell` to debug in real-time:
```bash
./dflow ci-shell
# Inside: cargo test --test integration_tests
```

### Trace Profiling
For performance bottlenecks, use `tokio-console` (requires compilation with special flags) to inspect task scheduling.
