---
title: Project Roadmap
label: kroki-rs.developer-guide.roadmap
---

# Project Roadmap

This page tracks the strategic evolution of **Kroki-rs**. For granular bug tracking, see [Technical Debt](#kroki-rs.developer-guide.tech-debt).

## 🟢 v0.0.5: Rust-Native Core (Completed)
- [x] Full Decoupling of PlantUML/CheerpJ.
- [x] Content-addressed CI Image partitioning.
- [x] Stable multi-arch distributions.

## 🟢 v0.0.6: Automated Release (Completed)
- [x] Professional `dflow` CLI orchestration.
- [x] Automated versioning & tagging.
- [x] Multi-file version verification.

## 🟢 v0.0.7: CI Performance (Completed)
- [x] Commit Statuses API integration.
- [x] sccache & clippy pre-warm optimizations.

## 🟢 v0.0.8: Docs & Distribution (Current)
- [x] MyST documentation CI integration.
- [x] Automated `release-reports.md`.
- [x] GitHub Run/Cache pruning maintenance.
- [x] Production OCI image smoke testing.

## 🔮 Next: v0.1.0 Modular Workspace
- [ ] Split into multi-crate Cargo workspace.
- [ ] Public Homebrew Tap.
- [ ] Official Plugin SDK.
- [ ] CI/Container redesign for deterministic caching and modular verification ([proposal](#kroki-rs.developer-guide.v010-ci-container-redesign)).
- [ ] Platform/repo strategy decision record for `devflow` + `kroki-rs-nxt` ([decision](#kroki-rs.developer-guide.v010-platform-migration-decision)).
- [ ] Detailed execution blueprint for phased implementation ([execution plan](#kroki-rs.developer-guide.v010-implementation-execution-plan)).
