#!/bin/bash
set -e

# src-scripts/ci-verify/repro-ci.sh
# Purpose: Run the full CI verification suite inside the production Docker environment.
# Optional: --version-check runs only Cargo.toml vs Git tag synchronicity (used by GHA release).
# Version check runs upfront during normal ci-verify; no separate verify-version.sh.
# Invoke from repo root (e.g. make cirun).

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

# --- Version synchronicity (Cargo.toml vs Git tag) ---
# Supports both local and GHA (GITHUB_REF). Skip if not on a version tag.
run_version_check() {
    REF=${GITHUB_REF:-$(git symbolic-ref -q HEAD 2>/dev/null || git describe --tags --always 2>/dev/null)}
    VERSION_TAG=""
    if [[ "$REF" == refs/tags/v* ]]; then
        VERSION_TAG=${REF#refs/tags/v}
    elif [[ "$REF" == v* ]]; then
        VERSION_TAG=${REF#v}
    fi
    if [ -z "$VERSION_TAG" ]; then
        echo "ℹ️ Not a version tag ($REF). Skipping version synchronicity check."
        return 0
    fi
    CARGO_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
    if [ "$CARGO_VERSION" != "$VERSION_TAG" ]; then
        echo "❌ Error: Version Mismatch!"
        echo "   Cargo.toml: $CARGO_VERSION"
        echo "   Git Tag:    $VERSION_TAG"
        echo "   Please update Cargo.toml to match the tag before releasing."
        return 1
    fi
    echo "✅ Version synchronization verified: v$CARGO_VERSION"
    return 0
}

# Mode: only version check (for GHA release workflow)
if [ "${1:-}" = "--version-check" ]; then
    run_version_check
    exit $?
fi

# Mode: interactive shell in CI container (same mounts as ci-verify for fast incremental fixes)
if [ "${1:-}" = "--shell" ]; then
    export DOCKER_BUILDKIT=1
    DOCKER_CMD=$(command -v podman || command -v docker)
    [ -z "$DOCKER_CMD" ] && { echo "❌ Error: Podman or Docker not found."; exit 1; }
    CI_FINGERPRINT=$(make -s print-base-fingerprint)
    CI_IMAGE_LOCAL="softmentor/kroki-rs-ci"
    CI_IMAGE_REMOTE="ghcr.io/softmentor/kroki-rs-ci:${CI_FINGERPRINT}"
    echo "📦 Ensuring CI image is ready (fingerprint: ${CI_FINGERPRINT})..."
    if $DOCKER_CMD pull "$CI_IMAGE_REMOTE" 2>/dev/null; then
        echo "✅ Pulled CI image from ghcr.io."
        $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL"
    else
        echo "📦 CI image not in registry; building locally..."
        $DOCKER_CMD build --target ci -t "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" -t "$CI_IMAGE_LOCAL" .
    fi
    echo "🐚 Opening shell in CI environment. Repo at /app (mounted); target at /app/target (volume)."
    echo "   Run 'make ghrun' or 'cargo test' for incremental fixes. Exit with 'exit'."
    exec $DOCKER_CMD run --rm -it \
        -v "$(pwd):/app" \
        -v "kroki-rs-target:/app/target" \
        -w /app \
        -e JOBS="${JOBS:-1}" \
        -e FEATURES="${FEATURES:-native-browser}" \
        -e PURGE_DISK \
        -e DEBUG_LOG \
        -e VERBOSE \
        -e NO_NETWORK \
        -e LOAD_TEST \
        --security-opt seccomp=unconfined \
        "$CI_IMAGE_LOCAL" \
        bash
fi

# --- Normal ci-verify flow: version check upfront, then container repro ---
run_version_check || true

CLEANUP=${CLEANUP:-false}
export DOCKER_BUILDKIT=1

# Capture target if provided (default to ghrun)
TARGET="ghrun"
if [[ $# -gt 0 && ! "$1" =~ ^-- ]]; then
    TARGET="$1"
    shift
fi

DOCKER_CMD=$(command -v podman || command -v docker)
if [ -z "$DOCKER_CMD" ]; then
    echo "❌ Error: Podman or Docker not found in PATH."
    exit 1
fi

echo "🚀 Using container engine: $DOCKER_CMD"

# Fingerprint (must match src-scripts/develop/vars/vars.mk and base-image.yml)
CI_FINGERPRINT=$(make -s print-base-fingerprint)
CI_IMAGE_LOCAL="softmentor/kroki-rs-ci"
CI_IMAGE_REMOTE="ghcr.io/softmentor/kroki-rs-ci:${CI_FINGERPRINT}"

echo "📦 Ensuring CI environment image is ready (fingerprint: ${CI_FINGERPRINT})..."
if $DOCKER_CMD pull "$CI_IMAGE_REMOTE" 2>/dev/null; then
    echo "✅ Pulled CI image from ghcr.io."
    $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL"
else
    echo "⚠️  CI image not in registry (${CI_FINGERPRINT})."
    if command -v gh >/dev/null 2>&1 && [ "$IS_CONTAINER" != "true" ]; then
        echo "🤔 Suggestion: Trigger remote build to populate GHCR for others?"
        # Only prompt if in a terminal
        if [ -t 0 ]; then
            read -t 5 -p "   Run 'gh workflow run base-image.yml'? (y/N): " trigger_remote || true
            if [[ "$trigger_remote" =~ ^[Yy]$ ]]; then
                gh workflow run base-image.yml
                echo "🚀 Remote build triggered. Proceeding with local build for current run..."
            fi
        fi
    fi
    echo "📦 Building CI image locally (target: ci)..."
    $DOCKER_CMD build --target ci -t "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" -t "$CI_IMAGE_LOCAL" .
fi

echo "🧪 Running CI target '$TARGET' inside container..."
$DOCKER_CMD run --rm \
    -v "$(pwd):/app" \
    -v "kroki-rs-target:/app/target" \
    -w /app \
    -e JOBS \
    -e PURGE_DISK \
    -e DEBUG_LOG \
    -e VERBOSE \
    -e NO_NETWORK \
    -e LOAD_TEST \
    --security-opt seccomp=unconfined \
    "$CI_IMAGE_LOCAL" \
    make $TARGET JOBS=${JOBS:-1} "$@"

if [ "$PURGE_DISK" = "true" ]; then
    echo "🧹 Cleaning up images and builder cache..."
    $DOCKER_CMD system prune -f
    $DOCKER_CMD volume rm kroki-rs-target || true
fi

echo "✅ Local CI verification (ci-verify) completed successfully."
