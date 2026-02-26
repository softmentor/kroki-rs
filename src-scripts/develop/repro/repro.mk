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
#           Configures Podman storage if PODMAN_STORAGE_DIR is set. Initializes Podman
#           machine if connection is not established (macOS/Linux).
#   devrun  Develop: (optional clean+prune) fmt, lint, build, test, smoke-test.
#   cirun   CI-verify: (optional clean+prune) then src-scripts/ci-verify/repro-ci.sh.
#   ghrun   Full pipeline: setup, then fmt, lint, build, test-ci, smoke-test, verify.
#   teardown  clean + docker-clean + prune + (podman-storage-clean if FULL_CLEANUP=true).
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
		if command -v podman >/dev/null 2>&1; then \
			echo "Checking Podman connection..."; \
			if ! podman system info >/dev/null 2>&1; then \
				echo "Removing broken Podman machine (if any)..."; \
				podman machine stop >/dev/null 2>&1 || true; \
				podman machine rm -f >/dev/null 2>&1 || true; \
				podman system connection rm podman-machine-default >/dev/null 2>&1 || true; \
				podman system connection rm podman-machine-default-root >/dev/null 2>&1 || true; \
				echo "Initializing Podman machine with $(VM_MEM)MB RAM and $(VM_CPUS) CPUs..."; \
				podman machine init --memory $(VM_MEM) --cpus $(VM_CPUS); \
				if [ -n "$(PODMAN_STORAGE_DIR)" ]; then \
					echo "Relocating VM disk images to external storage..."; \
					export PODMAN_STORAGE_DIR=$(PODMAN_STORAGE_DIR) && bash src-scripts/setup/podman-setup/podman-storage.sh; \
				fi; \
				echo "Starting Podman machine (this may take 30-60 seconds)..."; \
				podman machine start; \
				echo "✅ Podman machine ready"; \
			else \
				echo "✅ Podman connection established"; \
			fi \
		fi; \
		echo "Ensuring container base image is ready..."; \
		$(MAKE) docker-base; \
	else \
		echo "Inside container: skipping docker-base build."; \
	fi

.PHONY: devrun
devrun: $(_PRE_CLEAN)
ifeq ($(STEPS_PARALLEL),true)
	@echo "🚀 Running parallel local verification (fmt, lint, build)..."
	@$(MAKE) -j3 fmt lint build
else
	@$(MAKE) fmt lint build
endif
	@$(MAKE) test smoke-test
	@echo "✅ Local native development verification complete."

.PHONY: cirun
cirun: $(_PRE_CLEAN)
	@echo "🚀 Running container-based CI verification (ci-verify)..."
	@DEBUG_LOG="$(DEBUG_LOG)" VERBOSE="$(VERBOSE)" bash src-scripts/ci-verify/repro-ci.sh $(CI_ARGS)

.PHONY: cishell
cishell:
	@FEATURES="$(FEATURES)" JOBS="$(JOBS)" DEBUG_LOG="$(DEBUG_LOG)" VERBOSE="$(VERBOSE)" bash src-scripts/ci-verify/repro-ci.sh --shell

.PHONY: ghrun
ghrun: setup $(_PRE_CLEAN)
ifeq ($(STEPS_PARALLEL),true)
	@echo "🚀 Running parallel production verification (fmt, lint, build)..."
	@$(MAKE) -j3 fmt lint build
else
	@$(MAKE) fmt lint build
endif
	@$(MAKE) test-ci smoke-test verify
	@echo "🌟 Production-level verification (GitHub/Remote) complete."

.PHONY: teardown
teardown: clean
	@if [ "$(IS_CONTAINER)" != "true" ]; then \
		$(MAKE) docker-clean || true; \
		$(MAKE) prune || true; \
	fi
	@if [ "$(FULL_CLEANUP)" = "true" ]; then \
		$(MAKE) podman-storage-clean; \
		$(MAKE) local-temp-clean; \
		$(MAKE) gha-cache-prune; \
	fi
	@echo "🧹 Teardown complete. Disk space recovered."

.PHONY: gha-cache-prune
gha-cache-prune:
	@echo "🗑️  Pruning remote GHA caches..."
	@bash src-scripts/gh-tasks/prune-gha-cache.sh

.PHONY: podman-storage-clean
podman-storage-clean:
	@if [ -n "$(PODMAN_STORAGE_DIR)" ] && [ -d "$(PODMAN_STORAGE_DIR)" ]; then \
		echo "🗑️  Cleaning Podman storage at $(PODMAN_STORAGE_DIR)..."; \
		rm -rf "$(PODMAN_STORAGE_DIR)"/* || echo "⚠️  Warning: Could not fully clean Podman storage"; \
		echo "✅ Podman storage cleaned."; \
	else \
		echo "ℹ️  PODMAN_STORAGE_DIR not set or directory not found."; \
	fi

.PHONY: local-temp-clean
local-temp-clean:
	@echo "🗑️  Checking for local Chromium/temp bloat..."
	@if [ -n "$$(which getconf 2>/dev/null)" ]; then \
		TEMP_BASE=$$(getconf DARWIN_USER_TEMP_DIR 2>/dev/null); \
		if [ -n "$$TEMP_BASE" ]; then \
			BLOAT_DIR=$$(echo "$$TEMP_BASE" | sed 's/\/T\/$$/\/X\//')com.google.Chrome.code_sign_clone; \
			if [ -d "$$BLOAT_DIR" ]; then \
				echo "Found large Chromium clone at $$BLOAT_DIR. Cleaning..."; \
				rm -rf "$$BLOAT_DIR"; \
				echo "✅ Local temp bloat cleaned."; \
			fi \
		fi \
	fi

.PHONY: all
all: devrun

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
	@echo "  PURGE_DISK=true      Clean all caches before running"
	@echo "  DEBUG_LOG=true       Enable debug-level logging"
	@echo "  VERBOSE=true         Enable verbose tool output"
	@echo "  NO_NETWORK=true      Run in offline mode"
	@echo "  LOAD_TEST=true       Include high-concurrency load tests"
	@echo "  SECURITY_TEST=true   Run the full production/security integration suite"
	@echo "  JOBS=N               Limit INTERNAL build parallelism to N threads"
	@echo "  STEPS_PARALLEL=true  Run HIGH-LEVEL steps (fmt, lint, build) concurrently"
	@echo "  FULL_CLEANUP=true    Full cleanup including Podman storage (teardown only)"
	@echo "  FEATURES=\"\"           Build lean core without browser engine"
	@echo ""
	@echo "Examples:"
	@echo "  make devrun STEPS_PARALLEL=false"
	@echo "  make cirun LOAD_TEST=true JOBS=1"
	@echo "  make ghrun DEBUG_LOG=true VERBOSE=true"
	@echo "  make setup FEATURES=\"\""
	@echo "  make teardown FULL_CLEANUP=true"
