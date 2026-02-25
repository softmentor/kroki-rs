# src-scripts/develop/native/native.mk
#
# Purpose:
#   Defines all targets for the Rust lifecycle and local tooling: build, test,
#   lint, doc, clean, dist, verify, smoke-test, and related. Assumes vars.mk
#   is already included.
#
# Used by:
#   - Root Makefile (included after vars.mk)
#   - repro.mk targets (devrun, ghrun) depend on fmt, lint, build, test, etc.
#
# Targets:
#   build, release, build-ci, build-all, test, test-ci, test-v, lint, fmt, fix,
#   doc, clean, dist, verify, smoke-test, quick, bump, serve.
# ------------------------------------------------------------------------------

RELEASE_DIR = target/release
ifneq ($(CARGO_TARGET_DIR),)
    RELEASE_DIR = $(CARGO_TARGET_DIR)/release
endif
RELEASE_BIN = $(RELEASE_DIR)/$(BINARY_NAME)

ifneq (, $(shell which sccache 2>/dev/null))
export RUSTC_WRAPPER=sccache
endif

.PHONY: build
build:
	cargo build $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: release
release:
	cargo build --release $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: build-ci
build-ci:
	cargo build --release --all-targets $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: build-all
build-all: build-ci
	cargo clippy --release --all-targets $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS) -- -D warnings

.PHONY: test
test:
	cargo test --release $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS) $(TEST_FLAGS)

.PHONY: test-ci
test-ci:
	cargo nextest run --release --locked $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: test-v
test-v:
	cargo test --release $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS) -- --nocapture $(TEST_FLAGS)

.PHONY: lint
lint:
	cargo clippy --release --all-targets $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS) -- -D warnings
	cargo fmt --all -- --check

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fix
fix:
	cargo clippy --fix --allow-dirty --allow-staged $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: doc
doc:
	cargo doc --no-deps --document-private-items $(JOBS_FLAG) $(FEAT_FLAG) $(CARGO_FLAGS)

.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@if [ -d target ]; then \
		if [ "$(IS_CONTAINER)" = "true" ]; then \
			rm -rf target/* || echo "Warning: target/ is a mount point, cleaned contents only."; \
		else \
			cargo clean || rm -rf target; \
		fi \
	fi
	rm -rf $(DIST_DIR)

.PHONY: dist
dist: release
	@echo "Packaging $(BINARY_NAME) for $(PLATFORM)..."
	@mkdir -p $(DIST_DIR)
	@cp $(RELEASE_BIN) $(DIST_DIR)/
	@cd $(DIST_DIR) && tar -czvf $(ARCHIVE_NAME) $(BINARY_NAME)
	@cd $(DIST_DIR) && shasum -a 256 $(ARCHIVE_NAME) > $(ARCHIVE_NAME).sha256
	@echo "Distribution package created in $(DIST_DIR)/$(ARCHIVE_NAME)"

.PHONY: verify
verify: dist
	@echo "Verifying packaged binary..."
	@cd $(DIST_DIR) && ./$(BINARY_NAME) --version
	@echo "Running test conversion..."
	@$(DIST_DIR)/$(BINARY_NAME) convert -t d2 -f svg tests/fixtures/test.d2 > $(DIST_DIR)/test_output.svg
	@grep -q "<svg" $(DIST_DIR)/test_output.svg && echo "✅ Success" || (echo "❌ Failed"; exit 1)

.PHONY: smoke-test
smoke-test: release
	@echo "Starting native smoke test..."
	@PIDS=$$(lsof -ti :8000,8081); if [ -n "$$PIDS" ]; then echo $$PIDS | xargs kill -9 2>/dev/null || true; fi
	@$(RELEASE_BIN) serve > smoke-test.log 2>&1 &
	@sleep 3
	@curl --fail -s http://localhost:8081/health | grep '"status":"ok"' || (PIDS=$$(lsof -ti :8000,8081); if [ -n "$$PIDS" ]; then echo $$PIDS | xargs kill -9; fi; exit 1)
	@PIDS=$$(lsof -ti :8000,8081); if [ -n "$$PIDS" ]; then echo $$PIDS | xargs kill -9 2>/dev/null || true; fi
	@rm -f smoke-test.log
	@echo "✅ Smoke test passed!"

.PHONY: quick
quick: build test

.PHONY: bump
bump:
	@if [ -z "$(VERSION)" ]; then echo "Usage: make bump VERSION=x.y.z"; exit 1; fi
	@sed -i.bak '2,10s/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@cargo metadata --format-version=1 --all-features > /dev/null
	@sed -i.bak 's/logo_text: Kroki-rs V.*/logo_text: Kroki-rs V$(VERSION)/' docs/myst.yml
	@rm -f Cargo.toml.bak docs/myst.yml.bak

.PHONY: serve
serve:
	cargo run $(FEAT_FLAG) -- serve
