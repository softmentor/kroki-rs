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

## 3. Distribution (CD)
**Workflow**: `release.yml`
- Triggers only on version tags (`v*`).
- Orchestrates multi-platform binary builds and Docker pushes.
- Deploys documentation to GitHub Pages.

## Build Optimization

- **`sccache`**: Multi-arch compilation cache used as a secondary safety net across jobs.
- **`cargo-chef`**: Layered builds for super-fast dependency caching.
- **Host-Mount Caching**: The runner persists `.cargo-cache` and `target` directories between runs for maximum speed.

For deployment details, see [Deployments & Distribution](#kroki-rs.developer-guide.deployments).
