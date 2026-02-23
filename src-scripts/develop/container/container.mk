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
	$(CONTAINER_ENGINE) build --target base -t $(DOCKER_IMAGE_BASE):$(BASE_IMAGE_FINGERPRINT) -t $(DOCKER_IMAGE_BASE):latest .
else
	@echo "No container engine found (podman or docker). Skipping docker-base build."
endif

.PHONY: docker-build
docker-build:
	$(CONTAINER_ENGINE) build -t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest .

.PHONY: docker-multiarch
docker-multiarch:
	@if [ "$(CONTAINER_ENGINE)" = "*docker*" ]; then \
		docker buildx build --platform linux/amd64,linux/arm64 \
			-t $(DOCKER_IMAGE):v$(VERSION) -t $(DOCKER_IMAGE):latest \
			$(if $(BUILDX_PUSH),--push,--load) .; \
	else \
		echo "Building multi-arch images..."; \
		$(CONTAINER_ENGINE) build --platform linux/amd64,linux/arm64 \
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
