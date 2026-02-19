# Variables
BINARY_NAME=kroki-rs
DIST_DIR=dist
PLATFORM=$(shell uname | tr '[:upper:]' '[:lower:]')
ARCHIVE_NAME=$(BINARY_NAME)-$(PLATFORM).tar.gz

# Default target: complete project lifecycle
.PHONY: all
all: deps clean fmt lint build test doc verify

# Build in debug mode
.PHONY: build
build:
	cargo build

# Build in release mode
.PHONY: release
release:
	cargo build --release

# Run unit and integration tests
.PHONY: test
test:
	cargo test

# Run tests with output
.PHONY: test-v
test-v:
	cargo test -- --nocapture

# Linting
.PHONY: lint
lint:
	cargo clippy -- -D warnings
	cargo fmt --all -- --check

# Formatting
.PHONY: fmt
fmt:
	cargo fmt --all

# Generate code documentation
.PHONY: doc
doc:
	cargo doc --no-deps --document-private-items

# Clean build artifacts and dist
.PHONY: clean
clean:
	cargo clean
	rm -rf $(DIST_DIR)

# Package for distribution
.PHONY: dist
dist: release
	@echo "Packaging $(BINARY_NAME) for $(PLATFORM)..."
	@mkdir -p $(DIST_DIR)
	@cp target/release/$(BINARY_NAME) $(DIST_DIR)/
	@cd $(DIST_DIR) && tar -czvf $(ARCHIVE_NAME) $(BINARY_NAME)
	cd $(DIST_DIR) && shasum -a 256 $(ARCHIVE_NAME) > $(ARCHIVE_NAME).sha256
	@echo "Distribution package created in $(DIST_DIR)/$(ARCHIVE_NAME)"
	@cat $(DIST_DIR)/$(ARCHIVE_NAME).sha256

# Full verification: check, test, and package
.PHONY: verify
verify: lint test dist
	@echo "Verifying packaged binary..."
	@cd $(DIST_DIR) && ./$(BINARY_NAME) --version
	@echo "Running test conversion using release binary..."
	@$(DIST_DIR)/$(BINARY_NAME) convert -t d2 -f svg tests/fixtures/test.d2 > $(DIST_DIR)/test_output.svg
	@if grep -q "<svg" $(DIST_DIR)/test_output.svg; then \
		echo "✅ Verification Success!"; \
	else \
		echo "❌ Verification Failed!"; \
		exit 1; \
	fi

# Helper to run the server
.PHONY: serve
serve:
	cargo run -- serve

# Install node dependencies
.PHONY: deps
deps:
	npm install
