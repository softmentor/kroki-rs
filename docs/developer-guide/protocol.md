# Development & Release Protocol (Kroki-Flow)

This protocol ensures that `main` remains pristine, traceable, and always production-ready. All contributors (AI or human) must strictly follow these three phases for every change.

---

## Phase 1: Develop & Fix (Short-lived)
This phase is for rapid iterations on a specific requirement or bug.

| Step | Commands | Estimated Time | Cache vs Fresh | Load Test? |
| :--- | :--- | :--- | :--- | :--- |
| **1. Branch** | `git checkout -b feat/your-feature` | < 1 min | N/A | No |
| **2. Code** | Standard IDE development | Variable | N/A | No |
| **3. Test** | `make test` | ~10-20s | Incrementally Cached | No |
| **4. Docker** | `make docker-build && make docker-test` | ~30s - 10m | **Cache Heavy** (Base Image) | No |
| **5. Optimize** | `./scripts/fetch-binary.sh && make docker-pack` | < 10s | **Binary Injection** | No |

---

## Phase 2: Merge & Verification (Buffering)
This phase groups features into a versioned release branch for collective verification.

| Step | Commands | Estimated Time | Cache vs Fresh | Load Test? |
| :--- | :--- | :--- | :--- | :--- |
| **1. Release Br** | `git checkout -b v0.0.4` (from main) | < 1 min | N/A | No |
| **2. Buffer** | `git merge feat/* --squash` | < 1 min | N/A | No |
| **3. PR Gate** | Submit PR to `main` | ~2 min (CI) | **CI Cached** (Sccache) | Optional |
| **4. Fixes** | Commit directly to `v0.0.4` | Variable | Incremental | No |
| **5. Full Verify** | `make all` | ~5 min | Fresh/Incremental | **Yes (Manual)** |

---

## Phase 3: Release & Distribution (Production)
This phase officially moves verified code to production and triggers distribution artifacts.

| Step | Commands | Estimated Time | Cache vs Fresh | Load Test? |
| :--- | :--- | :--- | :--- | :--- |
| **1. Merge Main** | `git checkout main && git merge v0.0.4 --no-ff` | < 1 min | N/A (History only) | No |
| **2. Tag** | `git tag v0.0.4 && git push origin main --tags` | < 1 min | N/A | No |
| **3. CD Run** | Automated via `release.yml` | ~10 min | **Fresh Build** (Multi-arch) | **Yes (Verification)** |
| **4. Distribution** | Automated (Hex, Gh Releases, GHCR, Pages) | Automated | N/A | No |
| **5. Cleanup** | `git branch -d v0.0.4` | < 1 min | N/A | No |

---

## Governance Policies

### 1. Load Testing Policy
- **CI Smoke Tests**: Performed on *every PR* (Diagram rendering health).
- **Load Testing**: Mandatory *before* merging a `vX.X.X` branch to `main`. This involves running the server under high concurrency to verify `state.config.server.max_input_size` and browser pool stability.
- **Tools**: Use `artillery` or custom scripts (to be defined in `make load-test`).

### 2. Base Image Build Policy
**When to trigger `base-image.yml`:**
- Changes to `Dockerfile` stages preceding the `builder`.
- Updates to `package-lock.json` or `apt-get` system dependencies.
- **Freshness**: A manual fresh build should be triggered once per major release to ensure security patches are included.

- **CI**: Uses `Swatinem/rust-cache` and Docker Layer Caching (GHA type).
- **Invalidation**: Any change to `Cargo.lock` or `Dockerfile` will invalidate relevant caches.

### 4. Base Image Versioning & Lifecycle
To balance speed and reproducibility, we use a "Build on Change, Tag on Release" model:

- **Build on Change**: The base image is only physically rebuilt when `Dockerfile` or `package-lock.json` changes.
- **CI Usage**: Standard CI (`docker.yml`) uses `base:latest` for sub-minute builds.
- **Release Pinning**: On every official release (e.g., `v0.0.3`), the release workflow tags the current base image with that version (e.g., `:base-v0.0.3`). This ensures that if you need to patch an old release, you have the exact environment it was built in.

### 5. Hotfix & Emergency Patching Protocol
If a critical bug is found in production (`main`), use the following sequence:
1. **Branch off Tag**: `git checkout -b hotfix/v0.0.3-patch1 v0.0.3`
2. **Fix & Test**: Apply the minimal necessary patch and verify locally.
3. **Commit & Tag**: Commit the fix, bump the patch version in `Cargo.toml`, and tag it (e.g., `v0.0.3.1`).
4. **Push & Release**: `git push origin hotfix/v0.0.3-patch1 --tags` (Triggers `release.yml`).
5. **Backport**: Create a PR from `hotfix/v0.0.3-patch1` to `main` to ensure the fix is incorporated into the next major/minor release.

### 6. Version Synchronicity (Anti-Lying Release)
To prevent "lying releases" where the Git tag does not match the binary version:
- The `release.yml` pipeline strictly enforces a version check (`scripts/verify-version.sh`).
- If `Cargo.toml` version `!=` Git Tag, the release fails immediately.

---

## GitHub Infrastructure Configuration (Settings as Code)

To enforce the "Kroki-Flow" on GitHub, we use automated settings and branch protection.

| Tool | Purpose | Status |
| :--- | :--- | :--- |
| **[scripts/repo-settings.json](file:///Users/jinythattil/jt/code/softmentor/kroki-rs/scripts/repo-settings.json)** | JSON Template for Repository & Branch rules. | Mandatory |
| **[scripts/apply-repo-settings.sh](file:///Users/jinythattil/jt/code/softmentor/kroki-rs/scripts/apply-repo-settings.sh)** | Script to apply the template via `gh api`. | Admin Only |

### Required GitHub Settings
- **Merge Button**: Only "Create a merge commit" and "Squash merging" are allowed.
- **Branch Protection (`main`)**:
    - **Required Status Checks**: `clippy`, `fmt`, `test`, `smoke-test` (all from `docker.yml`).
    - **Require Pull Request Reviews**: At least 1 approval.
    - **No Force Pushes**: Ensures history stability.
    - **Merge Commits Allowed**: To preserve release history.

### How to apply changes:
```bash
chmod +x scripts/apply-repo-settings.sh
./scripts/apply-repo-settings.sh
```

---

## Appendix: Workflow Assessment (Kroki-Flow vs. Standards)

| Feature | Git-Flow | GitHub Flow | **Kroki-Flow (Ours)** |
| :--- | :--- | :--- | :--- |
| **Complexity** | High (Many long-lived branches) | Low (Main + Feature) | **Medium (Main + Release Buffers)** |
| **Grouping** | Via `develop` & `release/*` | Not supported (direct to `main`) | **Via `vX.X.X` branches** |
| **Safety** | High (Multiple stages) | Moderate (Relies on CI) | **High (PR-Gate + Release Staging)** |
| **Revertability** | Excellent (Merge commits) | Variable (Squash merges) | **Excellent (Merge commits to main)** |

**Assessment Notes**:
-   **Why we chose this**: We need the "bundling" power of Git-Flow (grouping features/fixes into a versioned release) but without the overhead of a permanent `develop` branch.
-   **PR-Gate Enforcement**: By requiring status checks on PRs, we ensure `main` never breaks.
-   **Traceability**: The `--no-ff` merge to `main` creates a clear record of every release.
