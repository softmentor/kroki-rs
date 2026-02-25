# src-scripts/develop/vars/vars.mk
#
# Purpose:
#   Central definition of environment variables, modifier flags, and the
#   content-addressable fingerprint for base/CI images. This file is included
#   first by the root Makefile; it defines no recipes (no targets with
#   commands), only variables and conditionals.
#
# Used by:
#   - Root Makefile (include)
#   - native.mk, container.mk, repro.mk (consume variables defined here)
#   - src-scripts/ci-verify/repro-ci.sh and .github/workflows/base-image.yml
#     must use the same fingerprint algorithm (SHA256 of Dockerfile).
#
# Fingerprint:
#   BASE_IMAGE_FINGERPRINT is the first 12 characters of SHA256(Dockerfile).
#   Only the Dockerfile is hashed because it is the sole file baked into the
#   image; all other inputs (Makefile, src-scripts/, install.sh) are
#   bind-mounted at runtime and do not affect the image contents.
#   Used to tag images as <name>:<fingerprint> and :latest.
# ------------------------------------------------------------------------------

# --- Project identity and paths ---
BINARY_NAME = kroki-rs
DIST_DIR = dist
PLATFORM = $(shell uname | tr '[:upper:]' '[:lower:]')
ARCHIVE_NAME = $(BINARY_NAME)-$(PLATFORM).tar.gz
VERSION ?= $(shell grep '^version =' Cargo.toml 2>/dev/null | head -n 1 | cut -d '"' -f 2)
RUST_VERSION := $(shell grep '^channel =' rust-toolchain.toml | cut -d '"' -f 2)

.PHONY: print-rust-version
print-rust-version:
	@echo $(RUST_VERSION)

.PHONY: print-container-engine
print-container-engine:
	@echo $(CONTAINER_ENGINE)

# --- Build configuration (override via make VAR=value) ---
FEATURES ?= native-browser
# JOBS: Controls internal build-level parallelism (e.g. cargo --jobs N).
JOBS ?=
# STEPS_PARALLEL: Controls high-level verification concurrency (e.g. running fmt, lint, build in parallel).
STEPS_PARALLEL ?= true

# --- Environment detection ---
PODMAN_STORAGE_DIR ?=
VM_MEM ?= 12288
VM_CPUS ?= 5
IS_CONTAINER ?= $(shell [ -f /.dockerenv ] || [ -f /run/.containerenv ] && echo true || echo false)

# --- Modifier flags ---
CARGO_FLAGS :=
TEST_FLAGS :=
FULL_CLEANUP ?= false

ifeq ($(PURGE_DISK),true)
    _PRE_CLEAN := clean prune
endif

ifeq ($(DEBUG_LOG),true)
    export RUST_LOG=debug
endif

ifeq ($(VERBOSE),true)
    CARGO_FLAGS += --verbose
    TEST_FLAGS += -- --nocapture
endif

ifeq ($(NO_NETWORK),true)
    CARGO_FLAGS += --offline
endif

ifeq ($(LOAD_TEST),true)
    TEST_FLAGS += --ignored
endif

ifneq ($(FEATURES),)
    FEAT_FLAG := --features $(FEATURES)
else
    FEAT_FLAG :=
endif

ifneq ($(JOBS),)
    JOBS_FLAG := --jobs $(JOBS)
else
    JOBS_FLAG :=
endif

# --- Content-addressable base image fingerprint ---
# Input: Dockerfile only. This is the sole file baked into the image; everything
# else (Makefile, src-scripts/, install.sh) is bind-mounted at runtime.
BASE_IMAGE_FINGERPRINT := $(shell openssl dgst -sha256 Dockerfile 2>/dev/null | sed 's/.* //' | cut -c1-12)

# --- Container image names ---
CONTAINER_ENGINE ?= $(shell which podman 2>/dev/null || which docker 2>/dev/null)
DOCKER_ORG = softmentor
DOCKER_IMAGE = $(DOCKER_ORG)/kroki-rs
DOCKER_IMAGE_BASE = $(DOCKER_IMAGE)-ci
DOCKER_IMAGE_CI = $(DOCKER_ORG)/kroki-rs-ci

# --- Scripting Utilities ---
.PHONY: print-base-fingerprint
print-base-fingerprint:
	@echo "$(BASE_IMAGE_FINGERPRINT)"

.PHONY: print-ci-image-local
print-ci-image-local:
	@echo "$(DOCKER_IMAGE_BASE)"

.PHONY: print-ci-image-remote
print-ci-image-remote:
	@echo "ghcr.io/$(DOCKER_IMAGE_BASE):$(BASE_IMAGE_FINGERPRINT)"
