---
title: CI/CD Pipelines
label: kroki-rs.developer-guide.pipelines
---

# CI/CD Pipelines

Kroki-rs uses a 3-tier pipeline architecture optimized for speed and atomic releases.

## 1. Continuous Integration (CI)
**Workflow**: `ci-build.yml`
- Runs on every PR.
- Executes inside the pre-built CI container for < 1 min feedback.
- Uses `cargo-nextest` and shared BuildKit caches.

## 2. Base & CI Image Management
**Workflow**: `base-image.yml`
- Triggers on infrastructure changes (Dockerfile, Makefile, etc.).
- Computes content-based fingerprints for immutable images.
- Pushes to GHCR.

## 3. Distribution (CD)
**Workflow**: `release.yml`
- Triggers only on version tags (`v*`).
- Orchestrates multi-platform binary builds and Docker pushes.
- Deploys documentation to GitHub Pages.

## Build Optimization

- **`cargo-chef`**: Layered builds for super-fast dependency caching.
- **Cache Mounts**: The Dockerfile uses `--mount=type=cache` for the Cargo registry and target.

For deployment details, see [Deployments & Distribution](#kroki-rs.developer-guide.deployments).
