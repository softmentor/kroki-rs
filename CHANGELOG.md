# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.1] - 2026-02-19

### Added
- **Core Engine**: Initial implementation of the high-performance Rust port of Kroki.
- **Server**: Axum-based HTTP API support for GET /:type/:format/:encoded_source.
- **CLI**: One-shot conversion tool with `kroki-rs convert`.
- **Supported Providers**:
  - Graphviz (SVG)
  - Mermaid (SVG, PNG, PDF)
  - PlantUML (SVG)
  - Vega/Vega-Lite (SVG)
  - WaveDrom (SVG)
  - BPMN (SVG, PNG)
  - D2 (SVG)
  - Ditaa (PNG)
- **Documentation**:
  - Comprehensive MyST-based documentation site.
  - Rustdoc API documentation integrated into the build.
  - Standardized `README.md`, `CONTRIBUTING.md`, and `SKILL.md` (LLM Integration Guide).
- **Tooling**:
  - Robust `Makefile` with `all`, `build`, `test`, `lint`, `doc`, `dist`, and `verify` targets.
  - GitHub Actions for automated GH Pages deployment and tagged releases.
  - Signed SHA-256 distribution artifacts.
- **Configuration**: `kroki.toml` for overriding system tool paths.
- **Branding**: Official project logos (light/dark themes).

### Changed
- Refactored `src` directory into a modular structure (`cli`, `server`, `config`, `utils`, `diagrams`).
- Extracted API handlers for better maintainability.

### Security
- Added automated SHA-256 checksum generation for distribution artifacts.
- Standardized `.gitignore` to prevent leakage of build artifacts.
