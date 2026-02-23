# Development & Release Protocol (Kroki-Flow)

This protocol ensures that `main` remains pristine, traceable, and always production-ready. All contributors (AI or human) must strictly follow these three phases for every change.

### Verification philosophy

**CI parity means running in the same environment as GitHub Actions:** the **CI container** (Docker base image + dependencies). The only way to get that is **container-based verification** (`./dflow ci-verify` locally, or `remote-ci.sh` for offload). Native runs on the host do *not* meet this objective and are not reproducible with CI. 

**Rule of Thumb**: 
1. Use `./dflow develop` for fast iteration.
2. Use `./dflow ci-verify` (mandatory) before pushing to ensure "passes locally ⇒ passes in CI."

---

## Phase 1: Develop & Fix (Feature Isolation)

This phase is for rapid iterations on a specific requirement or bug. **Feature branches MUST be cut from the latest `main` or a specific release branch.**

| Step | Commands | Estimated Time | Cache vs Fresh | Role |
| :--- | :--- | :--- | :--- | :--- |
| **1. Branch** | `git checkout main && git pull origin main && git checkout -b feat/your-feature` | < 1 min | N/A | Isolation |
| **2. Code** | Standard IDE development | Variable | N/A | Implementation |
| **3. Iteration** | `./dflow develop` | ~10-30s | **Warm** (Native) | Fast Feedback |
| **4. CI-verify** | `./dflow ci-verify` | **~45s** | **Warm** (Persistent Vol) | **CI parity (mandatory)** — same env as GHA |

---

## Phase 2: Pull Request & PR-Gate (Verification)

We use a **Stable Main** approach. No code enters `main` without passing established PR-Gates. 

| Step | Commands | Role |
| :--- | :--- | :--- |
| **1. Push** | `git push origin feat/your-feature` | Remote Sync |
| **2. Raised PR** | Open PR from `feat/your-feature` to `main` | Review Request |
| **3. PR-Gate** | Automated GHA Checks (`ci-build.yml`) | Automated Quality |
| **4. Review** | Peer review and approval | Human Quality |
| **5. Merge** | Squash Merge or Rebase into `main` | Integration |

> [!IMPORTANT]
> **PR Integrity**: Before raising a PR, ensure you have pulled the latest from `main` and passed a local containerized build (`./dflow ci-verify`). This guarantees the PR merge to `main` is clean.

---

## Phase 3: Release & Distribution (Production)

Release is a deliberate act after successful integration into `main`. 

| Step | Commands | Role |
| :--- | :--- | :--- |
| **1. Sync Main** | `git checkout main && git pull origin main` | Latest Source |
| **2. Verify Docs** | `cargo doc --no-deps` | Documentation Health |
| **3. Tagging** | `./src-scripts/gh-tasks/tag-release.sh` | Traceable Release |
| **4. CD Pipeline** | Automated via `release.yml` on Tag | Release Distribution |

> [!CAUTION]
> **Manual Tagging**: Only tag and push after verifying that `main` is stable and documentation builds correctly. The `release.yml` workflow will handle pushing to GHCR, GitHub Releases, and GH-Pages automatically.

---

## Governance Policies

### 1. Load Testing Policy
- **CI Smoke Tests**: Performed on *every PR* (Diagram rendering health).
- **Load Testing**: Mandatory *before* major releases. Use `./dflow ci-verify -t load` to verify browser pool stability under stress.

### 2. Base & CI Image Build Policy
- Changes to `Dockerfile`, `Makefile`, `install.sh`, or `src-scripts/**` trigger `base-image.yml`.
- This updates the fingerprinted images used by both GHA and local `./dflow ci-verify`.

### 3. Version Synchronicity
- The `release.yml` pipeline runs a version check before building.
- If `Cargo.toml` version `!=` Git Tag, the release fails. Use `make bump VERSION=X.Y.Z` before tagging.
