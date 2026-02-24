# src-scripts/develop/container/container.mk
#
# Purpose:
#   Defines targets for container engine (Docker/Podman) operations: building
#   the base image (with fingerprint and :latest tags), full app image,
#   multi-arch build, run, clean, and prune. Depends on variables from vars.mk.
#
# Used by:
#   - Root Makefile (included after native.mk)
#   - setup target in repro.mk calls docker-base; teardown calls docker-clean and prune.
#
# Targets:
#   docker-base, docker-build, docker-multiarch, docker-run, docker-clean, prune.
# ------------------------------------------------------------------------------

.PHONY: docker-base
docker-base:
ifneq ($(CONTAINER_ENGINE),)
	@if [ "$(IS_CONTAINER)" != "true" ]; then $(MAKE) docker-prune-legacy; fi
	@echo "📦 Pulling fingerprinted base from GHCR (fingerprint: $(BASE_IMAGE_FINGERPRINT))..."
	@$(CONTAINER_ENGINE) pull ghcr.io/$(DOCKER_IMAGE_BASE):$(BASE_IMAGE_FINGERPRINT) 2>/dev/null || \
	(echo "❌ Error: Base image $(BASE_IMAGE_FINGERPRINT) not found in GHCR." && \
	 echo "   Infrastructure is remote-first. Please ensure the 'Build Base Image' workflow" && \
	 echo "   has completed on GitHub for the current Dockerfile state." && exit 1)
else
	@echo "No container engine found (podman or docker). Skipping docker-base."
endif

.PHONY: docker-build
docker-build:
	@if [ "$(IS_CONTAINER)" != "true" ]; then $(MAKE) docker-prune-legacy; fi
	$(CONTAINER_ENGINE) build --build-arg RUST_VERSION=$(RUST_VERSION) -t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .

.PHONY: docker-pack
docker-pack:
	@if [ ! -f dist/$(BINARY_NAME) ]; then echo "❌ Error: dist/$(BINARY_NAME) not found. Build it first!"; exit 1; fi
	@echo "📦 Pulling fingerprinted base from GHCR (fingerprint: $(BASE_IMAGE_FINGERPRINT))..."
	$(CONTAINER_ENGINE) pull ghcr.io/$(DOCKER_IMAGE)-base:$(BASE_IMAGE_FINGERPRINT)
	$(CONTAINER_ENGINE) build --build-arg BASE_IMAGE=ghcr.io/$(DOCKER_IMAGE)-base:$(BASE_IMAGE_FINGERPRINT) -f Dockerfile.pack -t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .

.PHONY: docker-multiarch
docker-multiarch:
	@if [ "$(CONTAINER_ENGINE)" = "*docker*" ]; then \
		docker buildx build --platform linux/amd64,linux/arm64 \
			--build-arg RUST_VERSION=$(RUST_VERSION) \
			-t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest \
			$(if $(BUILDX_PUSH),--push,--load) .; \
	else \
		echo "Building multi-arch images..."; \
		$(CONTAINER_ENGINE) build --platform linux/amd64,linux/arm64 \
			--build-arg RUST_VERSION=$(RUST_VERSION) \
			-t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .; \
	fi

.PHONY: docker-run
docker-run:
	$(CONTAINER_ENGINE) run --rm -it -p 8000:8000 -p 8081:8081 $(DOCKER_IMAGE):latest

.PHONY: docker-clean
docker-clean:
	@if [ "$(IS_CONTAINER)" != "true" ] && [ -n "$(CONTAINER_ENGINE)" ]; then \
		$(CONTAINER_ENGINE) system prune -f; \
	else \
		echo "Skipping docker-clean inside container."; \
	fi

.PHONY: prune
prune:
	@if [ "$(IS_CONTAINER)" != "true" ] && [ -n "$(CONTAINER_ENGINE)" ]; then \
		$(CONTAINER_ENGINE) system prune -a --volumes -f; \
	else \
		echo "Skipping prune inside container."; \
	fi

.PHONY: docker-prune-legacy
docker-prune-legacy:
	@echo "🧹 Pruning legacy project images (keeping :$(BASE_IMAGE_FINGERPRINT) and :latest)..."
	@if [ -n "$(CONTAINER_ENGINE)" ]; then \
		$(CONTAINER_ENGINE) system prune -f; \
		IDS=$$($(CONTAINER_ENGINE) images --format "{{.Repository}}:{{.Tag}}" | grep "$(DOCKER_ORG)/kroki-rs" | grep -v "$(BASE_IMAGE_FINGERPRINT)" | grep -v "latest" || true); \
		if [ -n "$$IDS" ]; then \
			echo "Removing: $$IDS"; \
			echo "$$IDS" | xargs $(CONTAINER_ENGINE) rmi || true; \
		fi \
	fi
