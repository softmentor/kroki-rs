---
title: Release Artifacts & Installation
label: kroki-rs.user-guide.releases
---

# Release Artifacts & Installation

Kroki-rs is distributed through several channels to support different platforms and integration needs.

## 1. Installation Channels

| Channel | Method | Platform | Best For |
| :--- | :--- | :--- | :--- |
| **Install Script** | `curl \| sh` | Linux, macOS | Quickest setup, automatic PATH detection. |
| **Homebrew** | `brew install` | macOS, Linux | Managed updates and dependency handling. |
| **GitHub Releases** | Manual Download | All (Native) | Air-gapped envs or specific version pinning. |
| **Docker (GHCR)** | `docker pull` | All (OCI) | Server deployments and dependency-free usage. |

---

## 2. Install Script (Quickest)
The automated script detects your architecture and installs the optimized binary for your platform.

```bash
curl -sSfL https://raw.githubusercontent.com/softmentor/kroki-rs/main/install.sh | sh
```

---

## 3. Homebrew
For macOS and Linux users who prefer a package manager:

```bash
# Tap the repository
brew tap softmentor/kroki-rs

# Install the tool
brew install kroki-rs
```

---

## 4. GitHub Releases (Binaries)
Standardized native binaries are available for every release tag.

### Available Platforms
- `kroki-rs-x86_64-apple-darwin.tar.gz` (Intel Mac)
- `kroki-rs-aarch64-apple-darwin.tar.gz` (Apple Silicon)
- `kroki-rs-x86_64-unknown-linux-musl.tar.gz` (Generic Linux)
- `kroki-rs-aarch64-unknown-linux-musl.tar.gz` (ARM Linux)

### Verification
Always verify the integrity of your download using the provided `checksums.txt`:
```bash
shasum -a 256 -c checksums.txt
```

---

## 5. Docker Images
The official multi-arch image is hosted on **GitHub Container Registry (GHCR)**.

```bash
# Pull the latest version
docker pull ghcr.io/softmentor/kroki-rs:latest

# Or pin to a specific version
docker pull ghcr.io/softmentor/kroki-rs:v0.0.5
```

For detailed production instructions, see the [**Docker Deployment Guide**](#kroki-rs.user-guide.docker-deployment).
