# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2] - 2026-02-20

### Added
- **WebP Output Support**: Added high-quality, centralized compilation of SVGs and PNGs into the WebP format (`-f webp`), capable of perfect lossless vector rasterization.
- **WebP Configuration**: The ability to configure the WebP quality profile (`lossless`, `high`, `medium`, `low`, or `0-100`) via `kroki.toml` or the `--webp-quality` CLI flag.
- **Performance**: Introduced a local filesystem caching layer to instantly serve previously rendered diagrams (SHA-256 content-based hashing).
- **CLI Utilities**: Added a new `batch` command (`kroki-rs batch`) to recursively discover and convert all diagrams in a directory concurrently.
- **Supported Providers**: Added support for Excalidraw diagrams via the `excalidraw-to-svg` tool.
- **Testing**: Added dedicated integration tests for each individual provider to ensure reliable conversions.
- **Custom Font Loading**: Added the `--font` CLI argument and `fonts` configuration array to dynamically download and load external `.ttf` URLs for high-fidelity WebP and SVG rasterizations.
- **Async Execution**: Fully refactored the internal provider architecture to use `tokio::process` and `async-trait` for high-concurrency, non-blocking diagram generation (ADR 0003).
- **Adaptive Timeouts**: Implemented a dynamic timeout strategy that scales with payload size to protect against ReDoS and hung subprocesses.
- **Developer Experience**: Introduced the `define_provider!` macro and `run_process_with_timeout` helper to eliminate provider boilerplate and standardize error handling.
- **Documentation**: Standardized all documentation with MyST frontmatter, unique labels, and path-independent cross-references. Added site navigation for quick access to documentation sections.
- **Reference**: Added a project-wide Glossary and Abbreviations system.

### Changed
- **CI/CD Optimization**: Integrated `Swatinem/rust-cache@v2` into GitHub Actions to eliminate redundant compilation of the new image dependencies, keeping build times low.
- **Testing**: Refactored the integration test suite for better isolation, parallel execution, and targeted provider verification.

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
