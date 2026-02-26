---
title: Architecture Decision Records (ADRs)
label: kroki-rs.developer-guide.adr-index
---

# Architecture Decision Records (ADRs)

This section documents the critical architectural choices and design trade-offs made during the development of **Kroki-rs**.

## Active ADRs

| ADR | Title | Status |
| :--- | :--- | :--- |
| **[0001](#kroki-rs.adr.0001)** | Native WebP Conversion | Implemented |
| **[0002](#kroki-rs.adr.0002)** | Dynamic Font Loading | Implemented |
| **[0003](#kroki-rs.adr.0003)** | Async Subprocess & Adaptive Timeouts | Implemented |
| **[0004](#kroki-rs.adr.0004)** | Browser Instance Pooling & Recycling | Implemented |
| **[0005](#kroki-rs.adr.0005)** | Authentication & Authorization Model | Implemented |
| **[0006](#kroki-rs.adr.0006)** | Per-Provider Observability & Metrics | Implemented |
| **[0007](#kroki-rs.adr.0007)** | Custom Plugin API via Subprocess | Implemented |
| **[0008](#kroki-rs.adr.0008)** | Rust-Native Browser Automation | Implemented |
| **[0008.1](#kroki-rs.adr.0008.1)** | Browser Backend Evaluation | Implemented |
| **[0009](#kroki-rs.adr.0009)** | CI Optimization Strategy | Implemented |
| **[0010](#kroki-rs.adr.0010)** | Devflow Platform and kroki-rs-nxt Repository Strategy | Accepted |

---
> [!NOTE]
> ADRs are immutable once "Accepted." Significant changes to previous decisions should be recorded in a new ADR that references the original.
