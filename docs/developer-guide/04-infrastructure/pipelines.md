---
title: CI/CD Pipelines
label: kroki-rs.developer-guide.pipelines
---

# CI/CD Pipelines

Kroki-rs uses a 3-tier pipeline architecture optimized for speed and atomic releases.

## 1. Continuous Integration (CI)
**Workflow**: `ci-build.yml`
- Runs on every PR.
- **"Compile Once, Check Parallel"**: A single sequential `Build` job prepares artifacts, followed by three parallel verification jobs (`Lint & Format`, `Tests`, `Smoke & Verify`).
- Provides individual status bubbles on GitHub PRs for granular tracking.

## 2. Base & CI Image Management
**Workflow**: `base-image.yml` (Source of Truth)
- **Centralized Identity**: GitHub Actions is the master of Docker fingerprints and base images. Local tools pull directly from GHCR based on the `Dockerfile` hash.
- **Build-on-Demand**: The CI system automatically detects if the `Dockerfile` fingerprint changed and builds the multi-arch images inline if missing from GHCR.
- **Zero-Pull Caching**: Uses `actions/cache` to store the fingerprinted CI image as a `.tar` file for parallel job speed.

## CI Flow Architecture

### GitHub Actions Workflow (ci-build.yml)

```mermaid
sequenceDiagram
    participant PR as Pull Request / Tag
    participant Build as Job: Build (build-all)
    participant Cache as Actions Cache (Disk Sccache)
    participant Parallel as Jobs: Fmt/Lint/Test/Smoke

    PR->>Build: Trigger
    Build->>Cache: Restore .cargo-cache & target/ci
    Build->>Build: cargo build --all-targets (Pre-warm)
    Build->>Cache: Save .cargo-cache & target/ci
    Build->>Parallel: Trigger (FAN-OUT)
    
    rect rgb(240, 240, 240)
    Note over Parallel: Parallel verification using warm cache
    Parallel->>Cache: Restore (Read-Only)
    Parallel->>Parallel: cargo check / test / smoke
    end
    
    Parallel->>PR: Success/Failure status
```

### Local Reproducible Flow (repro-ci.sh)

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Script as repro-ci.sh
    participant Podman as Local Podman / Docker
    participant Registry as GHCR (Source of Truth)

    Dev->>Script: ./dflow ci-verify
    Script->>Script: Generate Fingerprint from Dockerfile
    Script->>Podman: Inspect local image:fingerprint
    
    alt Image exists locally
        Podman-->>Script: ✅ Match
    else Image missing
        Script->>Registry: Pull image:fingerprint
        Registry-->>Script: 📦 Pull complete
        Script->>Podman: Tag for persistent local reuse
    end
    
    Script->>Podman: Run Container (Mount target/ci, .cargo-cache)
    Podman->>Podman: Execute make ghrun
    Podman-->>Dev: Fast verification results
```

## 3. Distribution (CD)
**Workflow**: `release.yml`
- Triggers only on version tags (`v*`).
- Orchestrates multi-platform binary builds and Docker pushes.
- Deploys documentation to GitHub Pages.

## Build Optimization

- **`build-all` Pre-warming**: The initial CI job runs `cargo build --all-targets`. This populates the compilation cache for both the application and all test suites upfront, so subsequent parallel jobs only fetch from cache.
- **Disk-Based `sccache`**: To ensure maximum reliability across containerized environments, we use a disk-based cache (`.cargo-cache/sccache`) instead of the custom GHA backend. This shared directory is host-mounted to the container and preserved via `actions/cache`.
- **Target Isolation (`target/ci`)**: Containerized builds use a dedicated `target/ci` directory. This provides absolute isolation from native `target/` directories (e.g., on macOS hosts), preventing "Exec format errors" and redundant rebuilds.
- **Host-Mount Caching**: The runner persists `.cargo-cache` (registry, git, sccache) and `target/ci` directories between runs for maximum speed.

For deployment details, see [Deployments & Distribution](#kroki-rs.developer-guide.deployments).
