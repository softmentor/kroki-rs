# Tech Debts — Kroki-rs v0.0.2

Identified during pre-release code review. Ordered by impact × effort.

## 🔴 Critical

- [x] **TD-01**: `convert()` and `convert_to_file()` are 95% duplicated (`cli/mod.rs`)
- [x] **TD-02**: Font aggregation copy-pasted 4× (`main.rs`, `cli`, `handlers`)
- [x] **TD-03**: WebP format-routing copy-pasted 3× (`cli`, `handlers`)
- [x] **TD-13**: Timeout errors lose all context — no tool name, input size, or partial stderr (`diagrams/mod.rs`)
- [x] **TD-14**: Spawn errors are generic — don't identify which binary failed (`diagrams/mod.rs`)
- [ ] **TD-15**: Provider errors are opaque strings, not structured (`all providers`)
- [x] **TD-19**: No input size limits anywhere — unbounded memory risk (`handlers`, `cli`)

## 🟠 Major

- [x] **TD-04**: Capabilities re-discovered on every HTTP request (`handlers.rs`, `cli`)
- [x] **TD-05**: Server uses `println!` instead of `tracing` (`server/mod.rs`)
- [x] **TD-06**: Server discovers capabilities but discards them (`server/mod.rs`)
- [ ] **TD-07**: Configuration priority pattern not established (`all providers`)
- [x] **TD-16**: Server leaks internal errors to clients (`handlers.rs`)
- [x] **TD-17**: No partial stderr capture on timeout (`diagrams/mod.rs`)
- [ ] **TD-20**: No output size validation (`handlers`, `cli`)
- [x] **TD-21**: No format whitelist — arbitrary strings reach providers (`handlers.rs`)

## 🟡 Moderate

- [ ] **TD-08**: `WebpQuality` accepted but ignored (`image_converter.rs`)
- [x] **TD-09**: Cache dir resolution duplicated 3× (`cli`, `font_manager`)
- [x] **TD-10**: `cmd.rs` hardcodes `-Tsvg` ignoring format param
- [ ] **TD-11**: `validate()` is no-op on every provider
- [ ] **TD-12**: Verbose per-tool debug logging in `capabilities.rs`
- [x] **TD-18**: VegaLite pipeline errors don't identify which stage failed (`vega.rs`)
- [ ] **TD-22**: Unknown type (400) conflated with tool-not-installed (503) (`handlers.rs`)
- [ ] **TD-23**: `decode()` hides UTF-8 errors behind generic message (`utils/mod.rs`)
- [x] **TD-24**: `excalidraw.rs` ignores format parameter
- [x] **TD-25**: `bpmn.rs`, `wavedrom.rs`, `ditaa.rs` use blocking I/O in async
- [x] **TD-26**: `plantuml.rs` silently defaults to SVG for unknown formats
- [x] **TD-27**: Batch exits 0 even with partial failures (`cli/mod.rs`)

## 🔵 Minor

- [x] **TD-28**: Leftover TODO/exploratory comments (multiple files)
- [ ] **TD-29**: `test_decode_debug` uses println, never asserts (`utils/mod.rs`)
- [x] **TD-30**: `ditaa.rs` format validation has dead code branch
