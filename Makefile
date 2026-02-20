# Variables
BINARY_NAME=kroki-rs
DIST_DIR=dist
PLATFORM=$(shell uname | tr '[:upper:]' '[:lower:]')
ARCHIVE_NAME=$(BINARY_NAME)-$(PLATFORM).tar.gz
VERSION ?= $(shell grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)

# Default target: complete project lifecycle
# Uses nextest for fast parallel testing, then full verify (release build + dist)
.PHONY: all
all: deps clean fmt lint test-ci doc verify test-load

# Check for sccache and configure it if available (speeds up local builds)
ifneq (, $(shell which sccache))
export RUSTC_WRAPPER=sccache
endif

# Build in debug mode
.PHONY: build
build:
	cargo build

# Build in release mode
.PHONY: release
release:
	cargo build --release

# Run unit and integration tests (in release mode to share artifacts with dist)
.PHONY: test
test:
	cargo test --release

# Fast CI tests using nextest (debug profile, parallel execution)
.PHONY: test-ci
test-ci:
	cargo nextest run --locked

# Run tests with output
.PHONY: test-v
test-v:
	cargo test --release -- --nocapture

# Run heavy load/concurrency tests locally (skipped in CI)
.PHONY: test-load
test-load:
	cargo test --release -- --ignored --nocapture

# Linting
.PHONY: lint
lint:
	cargo clippy --release -- -D warnings
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

# Version bump helper
# Usage: make bump VERSION=0.0.3
.PHONY: bump
bump:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make bump VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "Bumping version to $(VERSION)..."
	@sed -i.bak '2,10s/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@sed -i.bak 's/"version": ".*"/"version": "$(VERSION)"/' package.json
	@sed -i.bak 's/logo_text: Kroki-rs V.*/logo_text: Kroki-rs V$(VERSION)/' docs/myst.yml
	@rm -f Cargo.toml.bak package.json.bak docs/myst.yml.bak
	@echo "Version bumped to $(VERSION). Please verify git diff before committing."

# Helper to run the server
.PHONY: serve
serve:
	cargo run -- serve

# Install node dependencies
.PHONY: deps
deps:
	npm install

# Container Engine (Podman preferred for Local/Mac)
CONTAINER_ENGINE ?= $(shell which podman 2>/dev/null || which docker 2>/dev/null)
DOCKER_IMAGE = kroki-rs

# Build Docker Base image (Dependencies only)
.PHONY: docker-base
docker-base:
	@echo "Building Docker base image (dependencies only)..."
	$(CONTAINER_ENGINE) build --target base -t $(DOCKER_IMAGE)-base:latest .

# Build Docker image (Standard: compile inside container)
.PHONY: docker-build
docker-build:
	@echo "Building Docker image $(DOCKER_IMAGE):v$(VERSION) (from source)..."
	$(CONTAINER_ENGINE) build -t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .

# Fast Pack Docker image (Uses pre-built binary in dist/)
.PHONY: docker-pack
docker-pack: dist
	@echo "Building Docker image $(DOCKER_IMAGE):v$(VERSION) (leveraging pre-built binary)..."
	$(CONTAINER_ENGINE) build -t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .

# Run Docker container locally
.PHONY: docker-run
docker-run:
	@echo "Running Kroki-rs container on http://localhost:8000 (Admin on 8081)..."
	$(CONTAINER_ENGINE) run --rm -it -p 8000:8000 -p 8081:8081 $(DOCKER_IMAGE):latest

# Verify Docker container functionality
.PHONY: docker-test
docker-test:
	@if [ -z "$$($(CONTAINER_ENGINE) images -q $(DOCKER_IMAGE):latest)" ]; then \
		echo "❌ Error: Image $(DOCKER_IMAGE):latest not found locally."; \
		echo "Please run 'make docker-build' or 'make docker-pack' first."; \
		exit 1; \
	fi
	@echo "Starting test container..."
	@$(CONTAINER_ENGINE) run -d --name kroki-test -p 8000:8000 -p 8081:8081 $(DOCKER_IMAGE):latest
	@sleep 5
	@echo "Verifying Health Check..."
	@curl --fail -s http://localhost:8081/health > /dev/null && echo "✅ Health OK" || (echo "❌ Health Failed"; $(CONTAINER_ENGINE) stop kroki-test; $(CONTAINER_ENGINE) rm kroki-test; exit 1)
	@echo "Verifying Mermaid Rendering (with compressed payload)..."
	@curl --fail -s http://localhost:8000/mermaid/svg/eJxLL0osyFAIcbHmUnDU1bVzsgYALroEhg > /dev/null && echo "✅ Rendering OK" || (echo "❌ Rendering Failed"; $(CONTAINER_ENGINE) stop kroki-test; $(CONTAINER_ENGINE) rm kroki-test; exit 1)
	@$(CONTAINER_ENGINE) stop kroki-test
	@$(CONTAINER_ENGINE) rm kroki-test
	@echo "✅ Docker verification successful!"

# Clean up Docker artifacts
.PHONY: docker-clean
docker-clean:
	@echo "Pruning Docker/Podman system (containers and images)..."
	$(CONTAINER_ENGINE) system prune -f

# Run Local CI (GitHub Actions via act)
# Mimics the GitHub Actions environment: Build, Load, and Smoke Test
# --container-daemon-socket - : prevents act from mounting the Docker socket (fixes Podman on macOS)
.PHONY: ci-local
ci-local:
	@echo "Running GitHub Actions locally for CI-Build workflow..."
	@export DOCKER_HOST=unix://$(shell podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}' 2>/dev/null || echo "/var/run/docker.sock") && \
	act -W .github/workflows/ci-build.yml --container-daemon-socket - -s GITHUB_TOKEN=$${CR_PAT:-$${GITHUB_TOKEN}}

# Complete Docker Lifecycle: Build, Test, and Local CI Verification
.PHONY: docker-all
docker-all: docker-build docker-test ci-local
