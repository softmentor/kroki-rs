---
description: Kroki-Flow development protocol — mandatory steps for all code changes
---

# Kroki-Flow Development Protocol

**This workflow is MANDATORY for all code changes.** Follow these steps exactly.

## Before Writing Any Code

1. Read `docs/developer-guide/protocol.md` for full context
2. Create a feature branch: `git checkout -b feat/your-feature`

## Development Loop (Local First!)

// turbo-all

1. Make your code changes
2. Run format check:
   ```bash
   cargo fmt --all --check
   ```
3. Run clippy lint:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
4. Run tests locally with nextest:
   ```bash
   cargo nextest run --locked
   ```
5. Generate source code documentation (user, developer, source docs), update changelog:
   ```bash
   cargo doc --no-deps --document-private-items
   ```
6. If tests involve Docker, verify locally:
   ```bash
   make docker-build && make docker-test
   ```

> **CRITICAL**: You MUST verify all changes pass locally (steps 2-5) BEFORE pushing to any branch or opening a PR. Never push untested code to CI. The CI is a safety net, not a development tool.

## Full Local Verification

For a complete lifecycle check (before merging to release branch):
```bash
make all
```
This runs: `deps → clean → fmt → lint → test-ci (nextest) → doc → verify (release build + dist) → test-load`

## Committing & Pushing

7. Only after ALL local checks pass, commit:
   ```bash
   git add . && git commit -m "type(scope): description"
   ```
8. Push to the feature branch:
   ```bash
   git push origin feat/your-feature
   ```

## PR & Merge Process

9. Open a PR to `main` (or to a release branch like `v0.0.4`)
10. Wait for CI checks to pass (clippy, fmt, test, smoke-test)
11. PR requires at least 1 approving review before merge
12. Merge to `main` uses `--no-ff` merge commits for traceability

## Release Process

13. Create release branch: `git checkout -b v0.0.X` from `main`
14. Buffer features via squash merges into the release branch
15. Full verify: `make all` (includes load tests + doc generation)
16. Merge to main: `git merge v0.0.X --no-ff`
17. Tag: `git tag v0.0.X && git push origin main --tags`
18. Cleanup: `git branch -d v0.0.X`

## Key Rules

- **NEVER** push directly to `main`
- **ALWAYS** test locally before pushing (steps 2-5 above)
- **ALWAYS** use `--locked` flag in CI to catch dependency drift
- **ALWAYS** generate docs (`cargo doc`) during development and before releases
- The `Makefile` has two test targets:
  - `make test` — release mode, for local development (serial)
  - `make test-ci` — debug mode with nextest (parallel, used by CI and `make all`)
- `Cargo.toml` already has `opt-level = 3` for dependencies in dev/test profiles
- `Cargo.lock` is committed to the repo (required for `--locked` flag)