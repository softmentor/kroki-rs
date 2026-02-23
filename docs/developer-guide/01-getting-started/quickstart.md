---
title: Quick Start
label: kroki-rs.developer-guide.quick-start
---

# Quick Start

Welcome to the **Kroki-rs** developer community! This guide will get you from zero to your first contribution in minutes.

## 1. Prerequisites

- **Rust**: [Install rustup](https://rustup.rs/) (Stable channel).
- **Podman**: [Install Podman](https://podman.io/) (Recommended for containerized verification).
- **GitHub CLI**: [Install `gh`](https://cli.github.com/) (For PR management).

## 2. Setup

```bash
# Initialize tools and base images
./dflow setup
```

## 3. Rapid Iteration

```bash
# Run local verification on your host OS
./dflow develop
```

## 4. First Contribution Flow

1.  **Branch**: `git checkout -b feat/my-cool-feature`
2.  **Hack**: Implement your changes in `src/`.
3.  **Verify**: Run `./dflow ci-verify` to ensure container parity.
4.  **Propose**: `./src-scripts/gh-tasks/propose-release.sh`

For detailed guidance, see:
- [Architecture Overview](#kroki-rs.developer-guide.architecture)
- [Local Development Guide](#kroki-rs.developer-guide.local-dev)
- [Verification Protocols](#kroki-rs.developer-guide.workflow)
