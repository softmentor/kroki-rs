# ADR 0009: CI Build Optimization Strategy

## Status
Accepted

## Context
Rust compilation is notoriously slow, particularly when building from scratch in CI environments. As the `kroki-rs` project grows and introduces more features (e.g., `native-browser`), the CI cycle time has increased, impacting developer productivity and feedback loops.

We need a strategy that:
1.  **Reduces Local CI-verify Time**: Fast local verification via `./dflow ci-verify`.
2.  **Optimizes GitHub Actions**: Minimal compute usage and faster PR checks.
3.  **Ensures Portability**: Works across Docker, Podman, and GHA.
4.  **Supports Multi-Arch**: Builds for `linux/amd64` and `linux/arm64`.

## Trade-off Analysis

| Strategy | Speed (Cache Hit) | Portability | Pros | Cons |
| :--- | :--- | :--- | :--- | :--- |
| **cargo-chef** | Fast (Layer based) | High (Standard Docker) | Best for GHA; robust for workspaces. | Invalidates on any dependency change. |
| **BuildKit Cache** | Very Fast (Mount) | Medium (GHA requires setup) | Persists even if layers mismatch. | Can be tricky to share across different CI hosts. |
| **sccache** | Moderate (Network) | Very High (S3/Cloud) | Global cache across all PRs/Archs. | Linking is still a bottleneck; network overhead. |
| **Persistent Volume** | Instant (Local) | Low (Host-bound) | No upload/download; real-time reuse. | Not available in ephemeral CI workers. |

## Decision
We will implement a **Dual Caching Strategy**:
1.  **`cargo-chef`** as the primary layering mechanism in the `Dockerfile`. This ensures that dependencies are cached in standard Docker layers, which is highly effective and portable for GitHub Actions.
2.  **BuildKit Cache Mounts** (`--mount=type=cache`) for the Cargo registry and git folders. This provides a secondary speedup for both local and CI builds by persisting downloaded crates and git data even when the `Cargo.lock` changes.
3.  **Local Persistent Volume** for the `target` directory in `./dflow ci-verify`. This allows for near-instant re-runs locally by bypassing the slow linking phase.

## Consequences
- **Positive**: PR verification times are expected to drop by 50-70% on warm builds. Local the re-run cycle will be significantly faster.
- **Negative**: The `Dockerfile` becomes more complex with multiple stages (`planner`, `builder`, `runtime`). Initial "cold" builds may be slightly slower due to `cargo-chef` setup overhead.

## Implementation (2026-02-23): Content-addressable identity and GHA parity

- **Fingerprinting**: Base/CI images are tagged by a content hash of `Dockerfile`, `Makefile`, `install.sh`, and `src-scripts/` (`src-scripts/develop/vars/vars.mk`). Same algorithm used in `src-scripts/ci-verify/repro-ci.sh` and `.github/workflows/base-image.yml`.
- **Ruthless Identity**: GHA runs all CI jobs inside the fingerprinted CI container (`ghcr.io/<repo>-ci:latest`), so local repro and GitHub Actions share the same environment (Debian, same tooling).
- **Smart pull**: `repro-ci.sh` pulls by hash from ghcr.io first and builds locally only if the image is missing.
- **Makefile**: Modularized under `src-scripts/develop/` into `vars/vars.mk`, `native/native.mk`, `container/container.mk`, `repro/repro.mk`. Scripts live under `src-scripts/setup/` and `src-scripts/ci-verify/`.

## Benchmarks (Verified 2026-02-23)
- **Cold Rebuild**: 16m 7s (Full container and target directory construction)
- **Warm Rebuild (Code change only)**: 45s (~21x speedup via persistent `target` volume)
- **Dependency Change Rebuild**: TBD (Expected to be handled by `cargo-chef` layers)
