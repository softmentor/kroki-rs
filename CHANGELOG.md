# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.8] - 2026-02-25

### Added
- **Documentation Infrastructure**: Integrated Node.js and MyST into the CI container for self-contained, offline-capable verification.
- **Automated Doc Validation**: Integrated `make doc-myst` into the `ci-verify` flow (conditional in GHA) to prevent documentation regressions.
- **Package Distribution**: Introduced `publish-packages.yml` for automated production OCI image builds, multi-arch support, and diagram smoke testing.

### Changed
- **CI Architecture**: Consolidated the CI pipeline into a 3-job "Prep-Build-Verify" structure, reducing overhead and improving failure visibility via backgrounded parallel checks.
- **Cache Optimization**: Implemented capacity-aware GHA cache pruning to maintain high performance and avoid storage limits.

### Fixed
- **CI Reliability**: Finalized transition to Commit Statuses API and robust SHA alignment for PR checks.
- **PR Container Pulls**: Resolved "manifest unknown" errors by adding credentials and ensuring transient CI images are pushed to GHCR on-demand.
- **Repository Integrity**: Retroactively tagged `v0.0.6` and `v0.0.7` to maintain version history.

## [0.0.7] - 2026-02-25

> [!NOTE]
> No documentation site update was published for this release.

### Fixed
- **CI Reliability**: Unified CI image naming and resolved registry fetch logic mismatch.
- **Diagnostics**: Added proactive container engine connection checks and suggest `./dflow setup` on failure.
- **Workflow Syntax**: Standardized `release.yml` YAML conditions for better compatibility.

### Changed
- **Performance**: Aligned CI build profiles to Release and added Clippy pre-warming, significantly reducing fan-out job durations.
- **Documentation**: Expanded troubleshooting and prerequisite guides for developer onboarding.

## [0.0.6] - 2026-02-25

### Added
- **Release Orchestration**: Automated versioning and branch verification via `dflow`.
- **Enhanced Verification**: Multi-file version synchronization and automated release reporting.
- **Migration Cleanup**: Finalized removal of Playwright and transition to native Rust browser engine.

## [0.0.5-1] - 2026-02-23

### Fixed
- **Stabilization**: Resolved race conditions in the native browser engine during high-concurrency rendering.
- **Cross-Platform Fixes**: Improved ARM64 binary stability for Linux distributions.

## [0.0.5] - 2026-02-22

### Added
- **Native Browser Engine (v2)**: Fully stabilized pure-Rust `headless_chrome` implementation for Mermaid and BPMN, replacing Node.js/Playwright ecosystem.
- **Robust Font Integration**: Dynamic font injection and pixel-perfect SVG rendering with OS-agnostic hinting.
- **Professional Unified Workflow**: Redesigned `Makefile` with standardized targets (`devrun`, `cirun`, `ghrun`, `teardown`) and professional modifier flags (`PURGE_DISK`, `DEBUG_LOG`, `LOAD_TEST`).
- **Secure Remote CI**: Repository-driven synchronization with SSH Agent Forwarding for private repo verification on remote build servers.
- **Parallelized CI Architecture**: Redesigned `ci-build.yml` using a "Compile Once, Check Parallel" structure across four distinct jobs for instant PR status feedback.
- **Zero-Pull CI Startup**: Integrated `actions/cache` to store fingerprinted Docker images as tarballs, enabling near-instant job startup without pulling from GHCR.
- **Self-Healing Registry Sync**: Automated "Build-on-Demand" strategy that detects Dockerfile mismatches and builds/pushes multi-arch images inline.
- **sccache Safety Net**: Integrated multi-arch `sccache` as a secondary compilation cache to ensure build parity across local and remote environments.
- **Resource Recovery**: Automated pruning of container objects and native caches via `make teardown`.

### Changed
- **Memory Optimization**: Reduced linker memory requirements by 70%—configured `debug=0` for dev/test profiles to prevent OOM in resource-constrained environments.
- **Storage Strategy**: Relocated Podman machine storage to external volumes on macOS via `src-scripts/setup/podman-setup/podman-storage.sh`.
- **Documentation Overhaul**: Consolidated all developer workflows, CI/CD architectures, and contribution guides into a unified structure.
- **Infrastructure**: Standardized all verification scripts (`repro-ci.sh`, `remote-ci.sh`) to share identical containerized environments.

### Verification
- **rel/0.0.5 Verified** (2026-02-23): Cold repro and full diagram suite passed. See [docs/releases/rel-0.0.5-verified.md](docs/releases/rel-0.0.5-verified.md).

## [0.0.4] - 2026-02-21

### Added
- **Server Middleware**: Introduced config-gated middleware for API Key Authentication, Token-bucket Rate Limiting, and per-provider Circuit Breakers. (TD-18, ADR 0005)
- **Observability**: Added rich Prometheus metrics tagged by provider and format, including request duration, payload size, and circuit breaker states. (ADR 0006)
- **Custom Plugin API**: Enabled the registration of external rendering tools via simple configuration, allowing kroki-rs to be extended without core code changes. (TD-05, ADR 0007)
- **Discovery Page**: A modern, interactive service root page providing quick access to health, metrics, and registered providers.
- **CI/CD Optimization**: Refactored release workflows to prioritize native artifacts and high-velocity iteration, removing Docker-heavy dependencies.

### Changed
- **Documentation**: Migrated to a unified MyST-based structure with robust cross-referencing and automated GitHub Pages deployment.
- **Dependency Management**: Standardized system dependencies (pixman, cairo) for stable document builds on Linux and macOS.

## [0.0.3] - 2026-02-20

### Added
- **Admin Dashboard & Health Check**: Introduced a dedicated Admin server on port `8081` offering a UI Dashboard and an orchestration-friendly `/health` API endpoint. (TD-19)
- **Production OCI Images**: Created a multi-stage Debian Dockerfile containing `node`, `graphviz`, and `chromium` out-of-the-box. (TD-16)
- **Structured Error Handling**: Implemented a strongly typed `DiagramError` enum to replace raw string errors, returning precise HTTP status codes (400, 503, 504) based on fault type. (TD-15)

- **Playwright Browser Pool**: Replaced Node.js CLI wrappers with a highly efficient, persistent headless Chromium daemon using `generic-pool`. This virtually eliminates cold-start latency for Mermaid and BPMN diagrams and bounds memory usage via strictly enforced TTL request limits.

### Changed
- **Configuration Precedence**: Rearchitected `Config` to strictly obey `CLI Defaults > Environment Variables > kroki.toml > Fallbacks`. (TD-07)

## [0.0.2] - 2026-02-20

### Added
- **WebP Output Support**: Added high-quality, centralized compilation of SVGs and PNGs into the WebP format (`-f webp`), capable of perfect lossless vector rasterization.
- **WebP Configuration**: Configure the WebP quality profile (`lossless`, `high`, `medium`, `low`, or `0-100`) via `kroki.toml` or the `--webp-quality` CLI flag. (TD-08)
- **Performance**: Introduced a local filesystem caching layer to instantly serve previously rendered diagrams (SHA-256 content-based hashing).
- **CLI Utilities**: Added a new `batch` command (`kroki-rs batch`) to recursively discover and convert all diagrams in a directory concurrently. (TD-27)
- **Supported Providers**: Added support for Excalidraw diagrams via the `excalidraw-to-svg` tool. (TD-24)
- **Custom Font Loading**: Added the `--font` CLI argument and `fonts` configuration array to dynamically download and load external `.ttf` URLs (ADR 0002).
- **Async Execution**: Refactored internal architecture to use `tokio::process` and `async-trait` for non-blocking diagram generation (ADR 0003). (TD-25)
- **Adaptive Timeouts**: Dynamic subprocess management scaling with payload size to protect against ReDoS and hung processes. (TD-13)
- **Robustness & Validation**:
    - **Input/Output Limits**: Enforce `max_input_size` (1MB) and `max_output_size` (50MB) to protect server resources. (TD-19, TD-20)
    - **Provider Validation**: All 10 providers now perform source input validation before execution. (TD-11)
    - **Format Whitelist**: Strict enforcement of supported formats (`svg`, `png`, `pdf`, `webp`, `txt`). (TD-21, TD-26)
- **Error Reporting**:
    - **Granular Decoding**: Detailed errors for Base64 decoding, Zlib/Deflate decompression, and UTF-8 validation. (TD-23)
    - **Tool Discovery**: Improved error messages distinguishing between "unknown diagram type" and "tool not installed". (TD-22)
    - **Contextual Failures**: Errors now include tool names and contextual hints for resolution. (TD-14)
- **Developer Experience**:
    - **Coding Patterns**: New guide documenting best practices for process execution, providers, and logging.
    - **Tech Debt Tracking**: Formalized remediation process with 28 of 30 items resolved in this release.
    - **Logging**: Consolidated and structured capability discovery logs. (TD-12)
- **Documentation**: Standardized all documentation with MyST frontmatter, unique labels, and path-independent cross-references.

### Changed
- **CI/CD**: Integrated `Swatinem/rust-cache@v2` into GitHub Actions.
- **Integration Tests**: Refactor for better isolation and parallel execution. Added assertions to previously debug-only tests. (TD-29)
- **CLI Deduplication**: Centralized diagram generation logic to eliminate duplication between `convert` and `batch` commands. (TD-01, TD-02, TD-03, TD-09)
- **Server Health**: Replaced all `println!` calls with structured `tracing` logs. (TD-05, TD-06, TD-16)

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
