---
description: Kroki-Flow development protocol — mandatory steps for all code changes
---

# Kroki-Flow Development Protocol

**This workflow is MANDATORY for all code changes.** Follow these steps exactly.

## Verification philosophy

**CI parity = container only.** The only way to verify in the same environment as GitHub Actions is to run inside the **CI container** (`./dflow ci-verify`). Native runs on the host (`./dflow develop`) are for fast iteration and optional sanity only — they do *not* guarantee "passes locally ⇒ passes in CI."

## Before Writing Any Code

1. Create/Sync a feature branch: `git checkout main && git pull origin main && git checkout -b feat/your-feature`

## Development Loop (Local First!)

1. Make your code changes.
2. Run local iteration:
   ```bash
   ./dflow develop
   ```
   *Fast feedback (fmt, clippy, parallel tests) on your native OS.*
3. **CI parity (mandatory before push):**
   ```bash
   ./dflow ci-verify
   ```
   *Runs the pipeline inside the CI container — same environment as GHA.*

## Release Branch Verification (Bundling)

If you are preparing a versioned release (e.g., `release/v0.0.5`):

1. **Native Full Check**:
   ```bash
   ./dflow develop -p
   ```
   *Equivalent to `make all` with a full purge. Ensures a clean native state.*
2. **Local CI-verify**:
   ```bash
   ./dflow ci-verify
   ```
   *Incremental container check to ensure local parity before pushing.*
3. **Push to Remote Release Branch**:
   ```bash
   git push origin release/v0.0.5
   ```
4. **Raise PR against `main`**.
5. **Verify GH Run**: Ensure all GHA checks pass on the PR.

## Manual Tagging (After PR Success)

Once the PR to `main` is merged and the GH build succeeds:

1. Sync local `main`: `git checkout main && git pull origin main`
2. Run tagging utility:
   ```bash
   bash src-scripts/gh-tasks/tag-release.sh
   ```
   *This script verifies the main checkout and documentation build before tagging.*

## Key Rules

- **NEVER** push directly to `main`.
- **ALWAYS** run `./dflow ci-verify` before pushing.
- **ALWAYS** bump version in `Cargo.toml` before tagging (use `make bump VERSION=X.Y.Z`).
- **ALWAYS** use `--locked` flag in CI to catch dependency drift.