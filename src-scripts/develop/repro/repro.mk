# src-scripts/develop/repro/repro.mk
#
# Purpose:
#   High-level orchestration targets: setup, devrun (develop), cirun (ci-verify),
#   ghrun, teardown, and help. Composes vars, native, and container targets.
#
# Used by:
#   - Root Makefile (included last; help is default goal)
#   - dflow CLI maps commands to these targets (e.g. repro -> cirun, dev -> devrun).
#
# Targets:
#   setup   Ensure native tools and base image; calls docker-base when not in container.
#   devrun  Develop: (optional clean+prune) fmt, lint, build, test, smoke-test.
#   cirun   CI-verify: (optional clean+prune) then src-scripts/ci-verify/repro-ci.sh.
#   ghrun   Full pipeline: setup, then fmt, lint, build, test-ci, smoke-test, verify.
#   teardown  clean + docker-clean + prune.
#   all     Alias for devrun.
#   print-base-fingerprint  Echo BASE_IMAGE_FINGERPRINT for scripting.
#   help    Print usage and targets.
# ------------------------------------------------------------------------------

.PHONY: setup
setup:
	@echo "🔧 Setting up environment..."
	@if [ "$(PLATFORM)" = "darwin" ]; then \
		echo "Detected macOS. Checking native tools..."; \
		rustc --version || (echo "❌ Rust not found. Install from rustup.rs"; exit 1); \
	fi
	@if [ "$(IS_CONTAINER)" != "true" ]; then \
		echo "Ensuring container base image is ready..."; \
		$(MAKE) docker-base; \
	else \
		echo "Inside container: skipping docker-base build."; \
	fi

.PHONY: devrun
devrun: $(_PRE_CLEAN) fmt lint build test smoke-test
	@echo "✅ Local native development verification complete."

.PHONY: cirun
cirun: $(_PRE_CLEAN)
	@echo "🚀 Running container-based CI verification (ci-verify)..."
	bash src-scripts/ci-verify/repro-ci.sh

.PHONY: cishell
cishell:
	@FEATURES="$(FEATURES)" JOBS="$(JOBS)" bash src-scripts/ci-verify/repro-ci.sh --shell

.PHONY: ghrun
ghrun: setup $(_PRE_CLEAN) fmt lint build test-ci smoke-test verify
	@echo "🌟 Production-level verification (GitHub/Remote) complete."

.PHONY: teardown
teardown: clean docker-clean prune
	@echo "🧹 Teardown complete. Disk space recovered."

.PHONY: all
all: devrun

.PHONY: print-base-fingerprint
print-base-fingerprint:
	@echo "$(BASE_IMAGE_FINGERPRINT)"

.PHONY: help
help:
	@echo "Usage: make [TARGET] [VARIABLE=VALUE]..."
	@echo ""
	@echo "Kroki-rs Professional Build System"
	@echo ""
	@echo "Targets:"
	@echo "  setup      Initialize environment (setup phase)"
	@echo "  devrun     Local develop verification"
	@echo "  cirun      Containerized ci-verify (repro-ci)"
	@echo "  cishell    Interactive shell in CI container (incremental test fixes)"
	@echo "  ghrun      Production CI verification (GitHub Actions / Remote)"
	@echo "  teardown   Reclaim all build and container disk space"
	@echo ""
	@echo "Variables (Flags):"
	@echo "  PURGE_DISK=true   Clean all caches before running"
	@echo "  DEBUG_LOG=true    Enable debug-level logging"
	@echo "  VERBOSE=true      Enable verbose tool output"
	@echo "  NO_NETWORK=true   Run in offline mode"
	@echo "  LOAD_TEST=true    Include high-concurrency load tests"
	@echo "  JOBS=N            Limit parallelism to N threads (e.g., JOBS=2)"
	@echo "  FEATURES=\"\"        Build lean core without browser engine"
	@echo ""
	@echo "Examples:"
	@echo "  make devrun PURGE_DISK=true"
	@echo "  make cirun LOAD_TEST=true JOBS=1"
	@echo "  make ghrun DEBUG_LOG=true VERBOSE=true"
	@echo "  make setup FEATURES=\"\""
