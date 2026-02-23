# Build Pipeline & CI/CD Architecture

This guide covers the internals of the **kroki-rs** build pipeline, feature flags, and verification protocols for v0.0.5+.

## Feature Flags

Kroki-rs uses Cargo features to manage binary size and system dependencies.

| Feature | Description | Default | Dependencies |
| :--- | :--- | :--- | :--- |
| `native-browser` | Essential for Mermaid/BPMN rendering. | **Enabled** | Headless Chromium + libs |
| (empty) | Lean core with only Stable CLI providers (Graphviz, D2, Ditaa). | - | CLI tools only |

## Build Parallelism (`JOBS`)

Headless browser compilation (specifically `headless_chrome`) is resource-intensive. 
- **Variable**: `JOBS` in the `Makefile`.
- **Behavior**: Controls `cargo --jobs`. 
- **Recommendation**: Set `JOBS=1` or `JOBS=2` in memory-constrained environments.
- **Optimization**: To prevent OOM during linking, `Cargo.toml` is configured with `debug = 0` and `split-debuginfo = "unpacked"` for `dev` and `test` profiles. This drastically reduces the RAM required by the linker.
- **IMPORTANT**: The default Podman VM (2GB) is insufficient. You **must** increase the VM memory to at least **4GB**, but **6GB** is highly recommended for a smoother build experience when using the `native-browser` feature:
  ```bash
  podman machine stop
  podman machine set --memory 6144
  podman machine start
  ```

## Mandatory Verification Workflow

To ensure 100% CI pass rates, every developer **MUST** run the Podman-based reproduction script before merging to `main`. This guarantees your code works in the exact environment used by our production Docker images.

```bash
# Mandatory check (ci-verify)
bash src-scripts/ci-verify/repro-ci.sh
```

### Environment parity (local ci-verify vs GHA)

Local ci-verify and GitHub Actions use the **same fingerprinted CI image** for 100% environment parity:

- **Fingerprint**: A content hash of `Dockerfile`, `Makefile`, `install.sh`, and `src-scripts/` (see `src-scripts/develop/vars/vars.mk`, `BASE_IMAGE_FINGERPRINT`). Images are tagged as `ghcr.io/<org>/<repo>-ci:<hash>` and `:latest`.
- **Local**: `src-scripts/ci-verify/repro-ci.sh` computes the hash, tries to pull `ghcr.io/...-ci:<hash>` first, and builds locally only if the image is missing.
- **GHA**: `ci-build.yml` runs all jobs (fmt, clippy, test, smoke-test) inside `ghcr.io/<repo>-ci:latest`. `base-image.yml` builds and pushes both base and CI images on path changes (Dockerfile, Makefile, install.sh, src-scripts/).

Verification: after changes, run `./dflow ci-verify` locally and confirm CI passes on the PR; both use the same Debian-based container identity.

### Podman Resource & Storage Optimization

If your local host is running low on disk space, you can relocate all Podman data to an external volume:

1.  **Configure Environment**: To make this persistent on your Mac (available across all shells and scripts but ignored by CI), add the following to your `~/.zshrc` (or `~/.bash_profile`):
    ```bash
    export PODMAN_STORAGE_DIR="/Volumes/your-drive/podman"
    ```
    Then restart your terminal or run `source ~/.zshrc`.

2.  **Relocate**: Run the relocator tool once:
    ```bash
    bash src-scripts/setup/podman-setup/podman-storage.sh
    ```

3.  **Manual Modification (Reference)**:
    If you need to manually point to a different drive or revert to internal storage:
    ```bash
    # 1. Stop Podman
    podman machine stop
    
    # 2. Re-point Symlink (Example)
    rm ~/.local/share/containers/podman/machine
    ln -s /Volumes/new-drive/podman/machine ~/.local/share/containers/podman/machine
    
    # 3. Revert to Internal Storage
    rm ~/.local/share/containers/podman/machine
    mkdir -p ~/.local/share/containers/podman/machine
    # (Then move your data back from the external volume)
    ```

> [!NOTE]
> **Native Configuration vs. Symlink**: While Podman adheres to XDG specifications and allows changing `root` or `runroot` via `containers.conf`, relocating the *machine disk* specifically on macOS is most reliably handled by the symlink approach. This ensures that the redirection remains active across Podman upgrades and specifically targets the large VM images without affecting other container metadata.
2.  **Cleanup**:
    ```bash
    # To recover disk/memory after a run
    CLEANUP=true bash src-scripts/ci-verify/repro-ci.sh
    ```

## Available Scripts (`src-scripts/`)

Scripts are organized by phase (setup, develop, ci-verify). See `src-scripts/README.md`.

- **setup**: `podman-setup/podman-storage.sh`, `gh-setup/apply-repo-settings.sh`, `gh-setup/repo-settings.json`
- **develop**: Makefile fragments (vars, native, container, repro) included by the root Makefile
- **ci-verify**: `repro-ci.sh` (main CI verification; includes version check; `--version-check` for release workflow), `remote-ci.sh` (offload ci-verify to a remote host via SSH), `verify-outcomes.sh` (assert expected outcomes after each phase), `full-verify.sh` (run full chain with assertions)

### Verifying expected outcomes

After each `dflow` phase you can assert that expected outcomes hold. Use either the full chain or step-by-step:

**One-shot full verification (clean state through ci-verify):**
```bash
./src-scripts/ci-verify/full-verify.sh
# Or skip the long ci-verify step: ./src-scripts/ci-verify/full-verify.sh --skip-ci-verify
```

**Step-by-step (same assertions):**
```bash
./dflow teardown && ./src-scripts/ci-verify/verify-outcomes.sh teardown
./dflow setup    && ./src-scripts/ci-verify/verify-outcomes.sh setup
./dflow develop  && ./src-scripts/ci-verify/verify-outcomes.sh develop
./dflow ci-verify && ./src-scripts/ci-verify/verify-outcomes.sh ci-verify
```

| Phase | Critical checks |
| :--- | :--- |
| **teardown** | Disk caches purged: `dist/` removed; `target/` absent or empty (best-effort); container prune ran |
| **setup** | `rustc` available; image `softmentor/kroki-rs-base` exists |
| **develop** | `target/release/kroki-rs` (or debug) exists; fmt, lint, build, test, smoke-test completed |
| **ci-verify** | Image `softmentor/kroki-rs-ci` exists; repro-ci.sh exited 0 and printed success message |

## Verification objective

The development flow has one clear objective: **verify in a CI environment** — i.e. the same environment as GitHub Actions: **Docker base image + dependencies**, not the host OS. That is the only reproducible way to guarantee “passes locally ⇒ passes in CI.”

- **Meets the objective**: Running the pipeline **inside the CI container** (same base image + deps as GHA). Reproducible on any host (macOS or Linux) because the runtime is the container, not the host.

So for verification before a PR (or for remote offload), use **container-based** flows: **ci-verify** locally, **remote-ci.sh** for offload (it runs **repro-ci.sh** in a container on the remote), and GHA’s container jobs.

## Standardized Build & Verification Workflow

To minimize cognitive load and ensure consistency across development environments, **kroki-rs** uses the professional **`dflow`** CLI wrapper. It provides a POSIX-standard interface for the underlying build system.

### Usage Syntax
```bash
./dflow <command> [options]
```

### Unified Commands
- `./dflow setup`: Initializes tools and environment (Native or Container base).
- `./dflow develop`: **Local iteration** (alias: `dev`). Rapid native-os verification (macOS). Use during feature development.
- `./dflow ci-verify`: **CI verification** (alias: `repro`). Runs the suite inside Podman. Use to ensure production parity locally.
- `./dflow ci-shell`: **CI dev shell** (alias: `shell`). Interactive shell inside the CI container with repo and target volume mounted. Use for fast incremental test fixes: edit on host, run `cargo test` or `make ghrun` inside the container.
- `./dflow teardown`: **Cleanup** (alias: `clean`). Reclaims disk space by purging native caches and container objects.

### Professional Options (POSIX)
| Option | Long Form | Role |
| :--- | :--- | :--- |
| `-p` | `--purge` | Force clean all caches and container objects. |
| `-d` | `--debug` | Enable `RUST_LOG=debug` tracing for troubleshooting. |
| `-v` | `--verbose` | Enable verbose tool output (cargo --verbose). |
| `-n` | `--no-network` | Restrict builds to local resources (offline mode). |
| `-t load` | `--test=load` | Include high-concurrency performance and stress tests. |
| `-j <n>` | `--jobs=<n>` | Limit parallelism to `<n>` threads. |
| `--lean` | - | Build lean core without browser engine. |

### Examples
```bash
# Rapid local iteration with debug logging
./dflow develop -d

# Clean containerized verification with load testing
./dflow ci-verify -p -t load

# Remote offload (runs ci-verify in container on remote host)
REMOTE_HOST="user@remote" bash src-scripts/ci-verify/remote-ci.sh
```

---

## Where commands run (environment clarity)

Only **container-based** runs meet the verification objective (same environment as GHA). **`./dflow develop`** is for fast local iteration on your host OS; **`./dflow ci-verify`** and **ci-shell** run the pipeline inside the CI container on your machine. **Remote offload** uses **remote-ci.sh**, which runs **repro-ci.sh** (container) on the remote host. **GitHub Actions** runs **ci-build.yml** (container jobs).

| Where you run it | Runtime | What runs | Meets verification objective? |
| :--- | :--- | :--- | :--- |
| **Your machine** | macOS (native) | `./dflow develop` | No — local iteration only |
| **Your machine** | macOS/Linux + container (Debian CI image) | `./dflow ci-verify` | **Yes** — same env as GHA |
| **Your machine** | macOS/Linux + container (Debian CI image) | `./dflow ci-shell` | **Yes** — same env, interactive |
| **Remote host** | Remote + container (repro-ci.sh) | `remote-ci.sh` → **repro-ci.sh** | **Yes** — offload, still container-based |
| **GitHub Actions** | GHA runner + container | **ci-build.yml** (container jobs) | **Yes** — source of truth |

---

## Professional Remote CI Verification

If you have a powerful Linux server on your network or a cloud VM (GCP/AWS), you can offload resource-intensive compilation using `src-scripts/ci-verify/remote-ci.sh`.

### Repository-Driven Pull (Efficiency & Parity)
For v0.0.5+, we use a **Repository-Driven Pull** strategy:
1.  **Low Local Bandwidth**: The remote host clones/pulls code directly from GitHub, saving your local upload bandwidth.
2.  **Parity**: Direct git-cloning identifies hidden configuration issues or missing tracked files.

### Private Repository Authentication
To securely verify private repositories on remote machines without storing keys there, we use **SSH Agent Forwarding**:
1.  Ensure your SSH agent is running locally: `ssh-add -L`.
2.  The script uses `ssh -A` to forward your local session to the remote host.
3.  The remote `git clone/pull` will use your local keys for authentication automatically.

### Execution
```bash
REMOTE_HOST="user@192.168.1.50" bash src-scripts/ci-verify/remote-ci.sh
```

---

## SSH & Persistent Sessions (`tmux`)

To make remote runs resilient to network drops:

#### 1. Passwordless SSH Setup
```bash
ssh-keygen -t ed25519 -C "your-email@example.com"
ssh-copy-id user@remote-ip
```

#### 2. Persistent Runs
```bash
ssh user@remote-ip
tmux new -s ci-run
# Press 'Ctrl+b' then 'd' to detach. Reconnect with 'tmux a -t ci-run'.
```

## Docker Stages

The `Dockerfile` is optimized with multiple stages:
- **`base`**: Final production environment (stripped of dev tools).
- **`ci`**: Includes the full Rust toolchain for ci-verify (`repro-ci.sh`).
- **`builder`**: High-speed build stage for generating release binaries.
