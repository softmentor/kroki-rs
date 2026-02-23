---
title: Development Workflow
label: kroki-rs.developer-guide.workflow
---

# Development Workflow (Kroki-Flow)

This protocol ensures that `main` remains pristine, traceable, and always production-ready.

## Verification Philosophy

**CI parity means running in the same environment as GitHub Actions:** the **CI container**. 

1.  **Develop**: Use `./dflow develop` for fast iteration on your host.
2.  **Verify**: Use `./dflow ci-verify` (mandatory) before pushing.

## Phases of Development

### Phase 1: Feature Isolation
- Cut branch from `main`: `git checkout -b feat/your-feature`.
- Rapid iteration with `./dflow develop`.

### Phase 2: PR-Gate
- Push to remote and open a PR.
- Automated GHA Checks (`ci-build.yml`) must pass.
- Peer review required.
- Squash/Merge into `main`.

### Phase 3: Release
- Sync `main` locally.
- Run safety checks: `./src-scripts/gh-tasks/tag-release.sh`.
- Tag and push. Automated CD handles the rest.

For details on the tooling:
- [Local Development Guide](#kroki-rs.developer-guide.local-dev)
- [CI/CD Infrastructure](#kroki-rs.developer-guide.pipelines)
