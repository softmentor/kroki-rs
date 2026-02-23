# Developer Flow CLI Standards (dflow)

## Vision
The **`dflow`** standard aims to unify the developer experience across heterogeneous projects (Rust, Node.js, Python, etc.) by providing a consistent, POSIX-compliant CLI interface for common development life-cycle tasks.

## 1. Standard Command Set
Every `dflow` implementation should support the following primary commands:

- **`setup`**: Idempotent initialization of the development environment.
- **`dev`**: Local, rapid iteration suite (e.g., native builds, hot-reloading).
- **`ci`**: Clean reproduction in a production-parity environment (e.g., Docker/Podman). Use for verification before push.
- **`clean`**: Deep cleanup of all build artifacts and ephemeral resources.

## 2. Standard Flag Mapping (POSIX)
Common modifiers should map to consistent flags:

| Short | Long | Purpose |
| :--- | :--- | :--- |
| `-p` | `--purge` | Full cache/state reset before execution. |
| `-d` | `--debug` | Enable high-verbosity logging/tracing. |
| `-v` | `--verbose` | Enable internal tool stdout/stderr output. |
| `-n` | `--no-network` | Restricted offline mode. |
| `-t` | `--test` | Select specific test suites or types. |
| `-j` | `--jobs` | Concurrency/parallelism control. |

## 3. Architecture Tiers
- **Tier 1 (Wrapper)**: A project-specific script (Bash/Python) that bridges to existing tools like `make` or `npm`.
- **Tier 2 (Schema)**: A `dflow.yaml` file defining commands and flags for a generic runner.
- **Tier 3 (Supervisor)**: A native Rust-based binary that monitors execution, manages logs, and provides a unified terminal UI regardless of the underlying stack.

## 4. Cross-Project Compatibility
The `dflow` supervisor should detect the project type and automatically inject the correct backend:
- **Rust**: Bridges to `cargo` / `Makefile`.
- **Node.js**: Bridges to `npm` / `pnpm`.
- **Python**: Bridges to `pytest` / `poetry`.

---
*Status: Proposal / Vision*
*Author: Antigravity*
