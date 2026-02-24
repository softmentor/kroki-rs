---
title: Packaging & Distribution
label: kroki-rs.developer-guide.deployments
---

# Packaging & Distribution

This guide outlines how to package, distribute, and verify **Kroki-rs** artifacts.

## Distribution Channels

| Channel | Format | Recommended For |
| :--- | :--- | :--- |
| **Homebrew** | Formula (RB) | macOS & Linux Desktop users. |
| **GitHub Releases** | Binaries (Tar.gz) | Manual installation, air-gapped envs. |
| **Docker (GHCR)** | OCI Image | Server deployments, CI/CD, Kubernetes. |
| **Cargo (crates.io)** | Source | Rust developers and system integrators. |

## 1. Automated Packaging

Use the `Makefile` to generate standardized release artifacts:

```bash
# Build and package the binary with checksums
make dist
```

This creates a `dist/` directory containing:
- `kroki-rs-<platform>.tar.gz`
- `checksums.txt` (SHA-256)

## 2. GitHub Container Registry (Docker)

Images are pushed to `ghcr.io` for every release tag. We support multi-architecture builds:
- `linux/amd64`
- `linux/arm64`

See the [Environment Guide](#kroki-rs.developer-guide.environment) for deep dives into image optimization.

## 3. Security & Integrity

### Binary Integrity
Always publish and verify SHA-256 checksums. For manual verification:
```bash
shasum -a 256 kroki-rs-<platform>.tar.gz
```

### Signing
- **macOS Notarization**: Binaries should be notarized to avoid "Developer cannot be verified" warnings.
- **GPG**: Release tags and binaries are signed with GPG to prove authenticity.

### Reproducible Builds
All public artifacts are built via [GitHub Actions Automation](#kroki-rs.developer-guide.automation) using the same fingerprinted CI environment used during development, ensuring the published binary exactly matches the source.
