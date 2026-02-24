---
title: Engineering the Development Environment
label: kroki-rs.developer-guide.environment
---

# Engineering the Development Environment

To support high-performance Rust builds and complex diagram rendering, **Kroki-rs** utilizes a custom-engineered development environment based on Podman and bit-identical containerization.

## 1. Local Setup: Podman

### macOS (Homebrew)
Install Podman and initialize the machine with sufficient resources for memory-intensive Rust compilation (specifically for `headless_chrome` and the native browser pool).

```bash
brew install podman
# 12GB RAM and 5 CPUs are recommended for stable verification
podman machine init --memory=12288 --cpus=5 --disk-size=100
podman machine start
```

### Resource Allocation
If you experience `Signal 9` (OOM) or performance lag during builds, verify your resource allocation:
```bash
podman machine stop
podman machine set --memory 12288 --cpus 5
podman machine start
```

## 2. Image Engineering: Deterministic Parity

We use content-addressable versioning to ensure that every developer and every CI runner is in a bit-identical environment.

### The Image Fingerprint
The system's identity is derived from its `Dockerfile`.
- **Calculation**: Centralized in `vars.mk`. The first 12 characters of the SHA-256 hash of the `Dockerfile` are retrieved via `make -s print-base-fingerprint`.
- **Source of Truth**: GitHub Actions is the master of fingerprints.
- **Remote-First Strategy**: The build system strictly pulls fingerprinted images from GHCR. Local builds of the base image are disabled to prevent environmental drift.

### Toolchain Verification Guard
To ensure absolute parity, the `repro-ci.sh` script includes a build-time guard. It verifies that the project's required Rust version (from `rust-toolchain.toml`) matches the version baked into the container.

If a mismatch is detected, the build fails with instructions to update the `Dockerfile` and rebuild the base image, preventing "invisible" toolchain downloads during CI.
### Local Fingerprint Caching
The `repro-ci.sh` script automatically tags pulled images with their fingerprint locally. This enables instant container startup on subsequent runs by skipping network checks.

## 3. The `dflow` Toolsuite

The `dflow` script is the primary entry point for the infrastructure lifecycle.

| Command | Purpose |
| :--- | :--- |
| `./dflow setup` | Initializes the Podman VM and prepares the environment. |
| `./dflow teardown` | Removes containers/images. Use `-f` for a **1TB Deep Cleanup**. |
| `./dflow ci-verify` | Performs containerized CI check with toolchain validation. |
| `./dflow ci-verify <target>` | Runs a specific sub-target (e.g. `lint`, `test-ci`). |
| `./dflow ci-shell` | Opens an interactive bash shell inside the CI container. |
| `gh-tasks/trigger-base-build.sh` | Manually triggers the GHA base build if fingerprint is missing. |

### Target Isolation (`target/ci`)
To prevent "Exec format errors" caused by host/container artifact clobbering, all containerized builds use `target/ci`. The host's native `target/` directory remains untouched.

## 4. Debugging & Troubleshooting

### Interactive Shell
If a test fails only in the container, use the shell to debug:
```bash
./dflow ci-shell
# Inside the container:
make test-ci
```

### Common Issues
- **Stale Caches**: If your `Dockerfile` changes aren't reflected, run `./dflow ci-verify --purge`.
- **Permission Denied**: If mounting `.cargo-cache` fails on Linux, ensure your user has appropriate permissions or run with `podman unshare`.
