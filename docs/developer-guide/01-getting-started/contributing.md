---
title: Contributing to Kroki-rs
label: kroki-rs.developer-guide.contributing
---
# Contributing to Kroki-rs

Thank you for your interest in contributing to **Kroki-rs**! We welcome contributions from everyone.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/your-username/kroki-rs.git
    cd kroki-rs
    ```
3.  **Create a new branch** for your feature or fix:
    ```bash
    git checkout -b feature/my-awesome-feature
    ```

## Development Workflow

### Prerequisites

-   Rust (stable)
-   `podman` or `docker` (required for `cirun` and container builds)
-   Dependencies for diagrams you want to test (e.g., Graphviz)
-   **(Optional)**: `sccache` for accelerating local builds.
    ```bash
    cargo install sccache
    ```

### Unified Workflow Targets

To ensure consistency across local iteration and CI, we use the professional **`dflow`** CLI wrapper. **Please use these instead of raw cargo commands whenever possible.**

| Command | Role | When to use |
| :--- | :--- | :--- |
| `./dflow setup` | **Environment Init** | After cloning or when tools are missing. |
| `./dflow develop` | **Local Verification** | Rapid iteration on your native OS (macOS). (alias: `dev`) |
| `./dflow ci-verify` | **CI Verification** | Before pushing, verify in the CI container (same env as GHA). (alias: `repro`) |
| `./dflow teardown` | **Disk Cleanup** | Purge all native and container caches. (alias: `clean`) |

**Concrete Examples:**
```bash
./dflow develop -p -v       # Full native verification with clean cache and verbose logs
./dflow ci-verify --test load   # Containerized CI verification with load tests
./dflow develop -d           # Local iteration with debug tracing enabled
```

### Pull Request Process

### Pull Request Process

1.  **Sync with Release Branch**: Before starting or submitting, ensure your feature branch is up-to-date with its parent `rel/vX.X.X`.
    ```bash
    git fetch origin rel/vX.X.X && git merge origin/rel/vX.X.X
    ```
2.  **Iterate & Verify**: Use `./dflow develop` for native OS checks and `./dflow ci-verify` for container parity (~45s warm rebuild).
3.  **Final Freshness Check**: Before merging your feature into the **release branch** (e.g., `rel/vX.X.X`), run a fresh check:
    ```bash
    DEBUG_LOG=false make all
    ```
4.  **Submit PR**: Open your PR from `feat/your-feature` to `rel/vX.X.X`. Once the release branch is ready, a final PR will be raised from `rel/vX.X.X` to `main`.

## Environmental Roles

| Environment | OS / Runtime | What runs | Role |
| :--- | :--- | :--- | :--- |
| **Local Dev** | macOS (native) | `./dflow develop` | Rapid iteration |
| **Local CI verification** | macOS/Linux + container | `./dflow ci-verify` | Verify in same env as GHA before pushing |
| **Remote offload** | Remote host + container | `remote-ci.sh` → repro-ci.sh | Offload; still container-based (reproducible) |
| **GitHub CI** | GHA runner + container | ci-build.yml (container jobs) | Source of truth |

---

## Adding a New Diagram Provider

1.  Create a new module in `src/diagrams/providers/`.
2.  Implement the `DiagramProvider` trait.
3.  Register the provider in `src/diagrams/registry.rs`.
4.  Add a test case/verification step.
5.  Update the documentation.

## Code of Conduct

Please be respectful and kind to others. Harassment or abusive behavior will not be tolerated.
