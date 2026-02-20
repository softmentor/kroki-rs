---
title: "ADR 0001: Native WebP Conversion"
label: kroki-rs.adr.0001
---
# ADR 0001: Native WebP Conversion

## Context
Kroki-rs needed to support the WebP image format to provide modernized, efficient diagram rendering suitable for modern web applications. The existing Java implementation heavily relied on external CLI tools or JVM-specific libraries to convert outputs.

## Decision
We chose to implement native Rust-based WebP conversion by leveraging the `resvg` crate for high-fidelity SVG rasterization and the `image` crate for WebP encoding. This approach bypasses external dependencies entirely.

## Trade-offs
- **Pros:** 
  - Zero external system dependencies (no need to install ImageMagick or `cwebp`).
  - Extremely fast due to parallelized native execution.
  - Predictable quality and memory safety natively enforced by Rust.
- **Cons:** 
  - Adds build time due to heavy graphic dependencies like `usvg` and `resvg`.
  - Binary size increases slightly.

## Status
Accepted and Implemented.
