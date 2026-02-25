---
title: Packaging & Distribution
label: kroki-rs.developer-guide.deployments
---

# Packaging & Distribution

This guide outlines how to package, distribute, and verify **Kroki-rs** artifacts.

## 1. Release Flow

Kroki-rs follows a binary release flow coordinated by `dflow`:

1.  **Proposal**: Run `./dflow release -b`. This uses `src-scripts/gh-tasks/propose-release.sh` to bump the version in `Cargo.toml`, update `CHANGELOG.md`, and create a `release/vX.X.X` PR branch.
2.  **Verification**: GitHub Actions runs the full `CI-Build` verification on the release PR.
3.  **Approval**: Once checks pass, the maintainer merges the PR.
4.  **Tagging**: Run `./dflow release --tag`. This uses `src-scripts/gh-tasks/tag-release.sh` to create an official Git tag and push it, triggering the distribution workflows (`release.yml` and `publish-packages.yml`).

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

Images are published to `ghcr.io` for every release tag via the `publish-packages.yml` workflow. This workflow builds the final production image and verifies its integrity before pushing.

### Production Image Structure
The production image (`ghcr.io/softmentor/kroki-rs`) is designed for stability and minimal size:
- **Runtime-only**: Contains basic system dependencies (Graphviz, D2, Chromium) but excludes the Rust toolchain and CI utilities.
- **Compiled Binary**: Includes the `kroki-rs` binary pre-compiled for the target architecture.
- **Multi-arch Support**: Published for both `linux/amd64` and `linux/arm64`.

### Automated Smoke Testing
Before any image is published to GHCR, it must pass a rigorous smoke test suite that:
1.  **Health Check**: Ensures the server starts and responds on port 8081.
2.  **Version Verification**: Confirms the reported version matches the release tag.
3.  **Rendering Validation**: Performs live diagram generation tests (Graphviz, D2) inside the fresh container to ensure all native drivers are correctly linked.

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
