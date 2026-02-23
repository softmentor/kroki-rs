---
title: "ADR 0003: Async Subprocess Execution and Adaptive Timeouts"
label: kroki-rs.adr.0003
---
# ADR 0003: Async Subprocess Execution and Adaptive Timeouts

## Context
Kroki-rs relies on external upstream dependencies (like Node.js scripts for Vega or BPMN) to generate specific diagram types. Some of these JS utilities suffer from Uncontrolled Resource Consumption (ReDoS) vulnerabilities. Passing complex or malformed strings can cause them to hang infinitely at 100% CPU. Previously, using `std::process::Command::wait` would indefinitely block the Rust server's executor threads, leading to a complete Denial of Service.

## Decision
We refactored the entire `DiagramProvider` architecture to be fully asynchronous using `#[async_trait]`. All external tool executions now use `tokio::process::Command` wrapped inside `tokio::time::timeout`. 
We also introduced an **adaptive timeout** strategy that calculates execution deadlines based on the payload size (starting at 3s and gracefully scaling to 10s based on diagram complexity), effectively eliminating the structural boilerplate across providers via compositional traits.

## Trade-offs
- **Pros:** 
  - Completely mitigates infinite-loop ReDoS vulnerabilities.
  - Async execution frees up Tokio worker threads, dramatically increasing concurrent throughput.
  - The `kill_on_drop(true)` strategy guarantees no orphan zombie processes are left lingering, securing the host memory.
  - Compositional design (using `define_provider!` macro and `run_process_with_timeout`) keeps adding new providers extremely lightweight.
- **Cons:** 
  - Added slight architectural complexity with boxed futures (`async-trait`).
  - Extremely large and complex, valid diagrams might occasionally cross the timeout threshold if the host machine is heavily loaded (mitigated by user-configurable timeout overrides).

## Status
Accepted and Implemented.
