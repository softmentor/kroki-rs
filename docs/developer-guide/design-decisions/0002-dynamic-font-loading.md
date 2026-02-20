---
title: "ADR 0002: Dynamic Font Loading"
label: kroki-rs.adr.0002
---
# ADR 0002: Dynamic Font Loading

## Context
When rendering SVGs to WebP or PNG using `resvg`, text elements require local fonts. If the server does not have the fonts requested by the diagram (e.g., custom Google Fonts), the text will fallback to system defaults, breaking the visual fidelity of carefully crafted diagrams like Excalidraw or Mermaid.

## Decision
We implemented a dynamic, just-in-time font downloading system. When a font URL is provided to the API or configuration:
1. The `reqwest` crate downloads the `.ttf` payload.
2. It is hashed (SHA-256) and securely cached in the local filesystem using the `dirs` crate.
3. The downloaded directory is dynamically injected into the `usvg` rendering context database.

## Trade-offs
- **Pros:** 
  - Users can request any font without requiring the host administrator to manually install system fonts.
  - Caching prevents repeated network requests, keeping rendering latency low.
- **Cons:** 
  - Initial rendering of a diagram with a new font incurs a network latency penalty.
  - Requires outgoing network access from the Kroki-rs server.

## Status
Accepted and Implemented.
