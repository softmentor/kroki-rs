---
title: "ADR 0010: Devflow Platform and kroki-rs-nxt Repository Strategy"
label: kroki-rs.adr.0010
---
# ADR 0010: Devflow Platform and kroki-rs-nxt Repository Strategy

## Status
Accepted (v0.1.0 planning)

## Context

Kroki-rs is entering two large parallel transformations:

1. A reusable developer workflow platform (`devflow` / `dwf`) that should work across multiple stacks.
2. A major product architecture evolution (multi-surface model with stronger module boundaries).

In-place refactoring of both tracks inside the current repository increases coupling risk, architecture drift, and governance complexity for open-source contributors.

## Decision

We will:

1. Create `devflow` as an independent Rust CLI repository and release it via Homebrew.
2. Create `kroki-rs-nxt` as a new repository for next-generation Kroki architecture.
3. Keep current `kroki-rs` as a stable maintenance line during migration.
4. Start `kroki-rs-nxt` as a single Cargo workspace monorepo; postpone multi-repo split until justified by ownership and release velocity.

## Alternatives Considered

### A) In-place refactor of `kroki-rs` + embedded workflow overhaul

- Pros: lower immediate repository overhead.
- Cons: mixed architecture states, slower refactor velocity, contributor confusion, higher compatibility burden.

### B) New `kroki-rs-nxt` + new `devflow` (selected)

- Pros: clean architecture runway, clear governance boundaries, reusable platform extraction.
- Cons: short-term coordination overhead across repositories.

## Rationale

- Separating `devflow` prevents Kroki-specific assumptions from leaking into a generic workflow platform.
- Separating `kroki-rs-nxt` allows deterministic architectural evolution without legacy coupling.
- Monorepo-first inside `kroki-rs-nxt` minimizes premature distribution complexity.

## Consequences

- We must maintain explicit migration docs and compatibility policy between `kroki-rs` and `kroki-rs-nxt`.
- CI and release governance must be defined per-repo with shared policy templates.
- Additional maintainer overhead is expected in the short term.

## Guardrails

- `devflow` must prove generic value via multi-stack examples before hard coupling to `kroki-rs-nxt`.
- `kroki-rs-nxt` phase gates must be measured (contract tests, conformance, CI parity) before feature migration continues.
- Any proposal to split `kroki-rs-nxt` into many repos requires a separate ADR.

## Related Documents

- [v0.1.0 Platform & Migration Decision Record](#kroki-rs.developer-guide.v010-platform-migration-decision)
- [v0.1.0 CI/Container Redesign Proposal](#kroki-rs.developer-guide.v010-ci-container-redesign)

## References

- Rust workspaces: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- Tauri ecosystem model: https://github.com/tauri-apps/tauri
- Terraform plugin model: https://developer.hashicorp.com/terraform/plugin/how-terraform-works#provider-plugins
- Nx plugin model: https://nx.dev/concepts/plugins/introduction
- GitHub reusable workflows: https://docs.github.com/actions/using-workflows/reusing-workflows
