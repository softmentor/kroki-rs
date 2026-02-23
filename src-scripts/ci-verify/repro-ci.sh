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
    export DOCKER_BUILDKIT=1
    DOCKER_CMD=$(make -s print-container-engine)
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
        RUST_VER=$(make -s print-rust-version)
        $DOCKER_CMD build --target ci --build-arg RUST_VERSION="$RUST_VER" -t "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" -t "$CI_IMAGE_LOCAL" .
    fi
    echo "🐚 Opening shell in CI environment. Repo at /app (mounted); target at /app/target (volume)."
    echo "   Cargo registry/git/sccache cached at .cargo-cache/ for fast incremental builds."
    echo "   Run 'make ghrun' or 'cargo test' for incremental fixes. Exit with 'exit'."
    mkdir -p "$(pwd)/.cargo-cache/registry" "$(pwd)/.cargo-cache/git" "$(pwd)/.cargo-cache/sccache"
    exec $DOCKER_CMD run --rm -it \
        -v "$(pwd):/app" \
        -v "$(pwd)/target:/app/target" \
        -v "$(pwd)/.cargo-cache/registry:/root/.cargo/registry" \
        -v "$(pwd)/.cargo-cache/git:/root/.cargo/git" \
        -v "$(pwd)/.cargo-cache/sccache:/root/.cache/sccache" \
        -w /app \
        -e JOBS="${JOBS:-1}" \
        -e FEATURES="${FEATURES:-native-browser}" \
        -e SCCACHE_DIR=/root/.cache/sccache \
        -e RUSTC_WRAPPER=sccache \
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

DOCKER_CMD=$(make -s print-container-engine)
if [ -z "$DOCKER_CMD" ]; then
    echo "❌ Error: Podman or Docker not found."
    exit 1
fi

echo "🚀 Using container engine: $DOCKER_CMD"

# Fingerprint (must match src-scripts/develop/vars/vars.mk and base-image.yml)
CI_FINGERPRINT=$(make -s print-base-fingerprint)
CI_IMAGE_LOCAL="softmentor/kroki-rs-ci"
CI_IMAGE_REMOTE="ghcr.io/softmentor/kroki-rs-ci:${CI_FINGERPRINT}"

echo "📦 Ensuring CI environment image is ready (fingerprint: ${CI_FINGERPRINT})..."
echo "🔍 Remote image: ${CI_IMAGE_REMOTE}"
if $DOCKER_CMD pull "$CI_IMAGE_REMOTE" 2>/dev/null; then
    echo "✅ Pulled CI image from ghcr.io."
    $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL"
else
    echo "⚠️  CI image not in registry (${CI_FINGERPRINT})."
    echo "📦 Building CI image locally (target: ci)..."
    RUST_VER=$(make -s print-rust-version)
    $DOCKER_CMD build --target ci --build-arg RUST_VERSION="$RUST_VER" -t "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" -t "$CI_IMAGE_LOCAL" .
fi

echo "🧪 Running CI target '$TARGET' inside container..."
mkdir -p "$(pwd)/target" "$(pwd)/.cargo-cache/registry" "$(pwd)/.cargo-cache/git" "$(pwd)/.cargo-cache/sccache"
$DOCKER_CMD run --rm \
    -v "$(pwd):/app" \
    -v "$(pwd)/target:/app/target" \
    -v "$(pwd)/.cargo-cache/registry:/root/.cargo/registry" \
    -v "$(pwd)/.cargo-cache/git:/root/.cargo/git" \
    -v "$(pwd)/.cargo-cache/sccache:/root/.cache/sccache" \
    -w /app \
    -e JOBS \
    -e PURGE_DISK \
    -e DEBUG_LOG \
    -e VERBOSE \
    -e NO_NETWORK \
    -e LOAD_TEST \
    -e SCCACHE_DIR=/root/.cache/sccache \
    -e RUSTC_WRAPPER=sccache \
    --security-opt seccomp=unconfined \
    "$CI_IMAGE_LOCAL" \
    make $TARGET JOBS=${JOBS:-1} "$@"

if [ "$PURGE_DISK" = "true" ]; then
    echo "🧹 Cleaning up images and builder cache..."
    $DOCKER_CMD system prune -f
    rm -rf "$(pwd)/target" "$(pwd)/.cargo-cache"
fi

echo "✅ Local CI verification (ci-verify) completed successfully."
