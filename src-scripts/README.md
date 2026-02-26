# src-scripts: Scripts and Makefile modules by phase

All operational scripts and Makefile fragments live under `src-scripts/`, organized by **development phase** for consistency with the Kroki-Flow protocol.

## Layout

| Phase       | Folder      | Purpose | Contents |
|------------|-------------|---------|----------|
| **Setup**  | `setup/`    | One-time or occasional: machine and repo configuration | `podman-setup/podman-storage.sh`, `gh-setup/apply-repo-settings.sh`, `gh-setup/repo-settings.json` |
| **Develop**| `develop/`  | Day-to-day build and test (Makefile fragments) | `vars/`, `native/`, `container/`, `repro/` |
| **CI-verify** | `ci-verify/` | Pre-merge verification; run in container | `repro-ci.sh` |
| **Release**| (GHA only)  | Release runs only on GitHub Actions; version check is run upfront in `repro-ci.sh` (use `--version-check` in release workflow) | — |

## Develop (Makefile)

The root `Makefile` includes, in order:

- `src-scripts/develop/vars/vars.mk` — variables and fingerprint
- `src-scripts/develop/native/native.mk` — Rust targets (build, test, lint, …)
- `src-scripts/develop/container/container.mk` — Docker/Podman targets
- `src-scripts/develop/repro/repro.mk` — setup, devrun, cirun, ghrun, teardown, help

**Fingerprint:** The same content hash (Dockerfile, Makefile, install.sh, `src-scripts/`) is computed in `develop/vars/vars.mk`, `ci-verify/repro-ci.sh`, and `.github/workflows/base-image.yml`. Keep the algorithm in sync so local ci-verify and GHA use the same image tags.

## Setup

- **podman-setup/** — Podman machine configuration: `podman-storage.sh` (relocate VM storage to an external volume).
- **gh-setup/** — GitHub repo configuration: `apply-repo-settings.sh` (apply branch protection and repo settings via `gh api`), `repo-settings.json` (template).

## CI-verify

- **repro-ci.sh** — Run full CI in the fingerprinted container. Optional upfront version check (Cargo.toml vs Git tag).

## Naming (protocol)

- **Setup**: environment and repo setup
- **Develop**: local build and test (`make devrun`, `./dflow develop`)
- **CI-verify**: containerized or remote CI reproduction (`make cirun`, `./dflow ci-verify`)
- **Release**: handled by GHA; version gate is part of ci-verify (`repro-ci.sh --version-check`)
