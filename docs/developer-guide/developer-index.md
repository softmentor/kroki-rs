---
title: Developer Guide
label: kroki-rs.developer-guide.developer-index
---
# Developer Guide

This guide is for developers who want to contribute to **Kroki-rs** or understand its internal architecture.

## What is Covered?

In this section, you will find:
- **[How to build and contribute](#kroki-rs.developer-guide.build-contribute)**: Deep dive into the Provider Pattern, project structure, and guidelines for adding new providers.
- **[How to package, release, deploy and operate](#kroki-rs.developer-guide.release-deploy)**: Instructions for building, packaging, and distributing release artifacts.
- [**Browser Rendering**](#kroki-rs.developer-guide.browser-rendering): Architecture of the native Rust browser engine and Java-free PlantUML integration.
- [**Coding Patterns**](#kroki-rs.developer-guide.coding-patterns): Established coding conventions — process execution, provider implementation, error handling, async I/O, and configuration.
- **[Development Protocol](#kroki-rs.developer-guide.protocol)**: Strict rules for branching, PRs, and atomic releases (Kroki-Flow).
- **[Docker Guide](#kroki-rs.developer-guide.docker)**: Build, test, debug, and operate Kroki-rs in OCI containers.
- **[CI/CD Infrastructure](#kroki-rs.developer-guide.ci-cd)**: High-performance pipeline for verification and atomic releases.
- **[Tech Debt Tracking](#kroki-rs.developer-guide.tech-debt-tracking)**: How we track, prioritize, and remediate technical debt across releases.
- **[Roadmap](#kroki-rs.roadmap)**: Future plans and scheduled features for Kroki-rs.
- **[Contributing](#kroki-rs.contributing)**: Guidelines on how to successfully propose changes and submit PRs.

## Architecture Decision Records (ADRs)

Key design decisions and architectural trade-offs are documented using the ADR format. Please review these before proposing major structural changes:

- [ADR 0001: Native WebP Conversion](#kroki-rs.adr.0001)
- [ADR 0002: Dynamic Font Loading](#kroki-rs.adr.0002)
- [ADR 0003: Async Subprocess Execution and Adaptive Timeouts](#kroki-rs.adr.0003)
- [ADR 0004: Browser Instance Pooling & Recycling](#kroki-rs.adr.0004)
- [ADR 0005: Authentication & Authorization Model](#kroki-rs.adr.0005)
- [ADR 0006: Per-Provider Observability & Metrics](#kroki-rs.adr.0006)
- [ADR 0007: Custom Plugin API via Subprocess Protocol](#kroki-rs.adr.0007)
- [ADR 0008: Rust-Native Browser Automation (Eliminating Node.js)](#kroki-rs.adr.0008)

If you are looking for information on how to use Kroki-rs, please proceed to the [User Guide](#kroki-rs.user-guide.user-index).
