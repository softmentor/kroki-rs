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
5. If tests involve Docker, verify locally:
   ```bash
   make docker-build && make docker-test
   ```

> **CRITICAL**: You MUST verify all changes pass locally BEFORE pushing to any branch or opening a PR. Never push untested code to CI. The CI is a safety net, not a development tool.

## Committing & Pushing

6. Only after ALL local checks pass, commit:
   ```bash
   git add . && git commit -m "type(scope): description"
   ```
7. Push to the feature branch:
   ```bash
   git push origin feat/your-feature
   ```

## PR & Merge Process

8. Open a PR to `main` (or to a release branch like `v0.0.4`)
9. Wait for CI checks to pass (clippy, fmt, test, smoke-test)
10. PR requires at least 1 approving review before merge
11. Merge to `main` uses `--no-ff` merge commits for traceability

## Release Process

12. Create release branch: `git checkout -b v0.0.X` from `main`
13. Buffer features via squash merges into the release branch
14. Full verify: `make all` (includes load tests)
15. Merge to main: `git merge v0.0.X --no-ff`
16. Tag: `git tag v0.0.X && git push origin main --tags`
17. Cleanup: `git branch -d v0.0.X`

## Key Rules

- **NEVER** push directly to `main`
- **ALWAYS** test locally before pushing (steps 2-5 above)
- **ALWAYS** use `--locked` flag in CI to catch dependency drift
- The `Makefile` has two test targets:
  - `make test` — release mode, for local development
  - `make test-ci` — debug mode with nextest, for CI speed
- `Cargo.toml` already has `opt-level = 3` for dependencies in dev/test profiles
