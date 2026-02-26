#!/bin/bash
set -e

# src-scripts/ci-verify/repro-ci.sh
# Purpose: Run the full CI verification suite inside the production Docker environment.
# Optional: --version-check runs only Cargo.toml vs Git tag synchronicity (used by GHA release).
# Version check runs upfront during normal ci-verify; no separate verify-version.sh.
# Invoke from repo root (e.g. make cirun).

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

# --- Environment Setup (Handle container-specific Git settings early) ---
if [ "$IS_CONTAINER" = "true" ]; then
    git config --global --add safe.directory "$(pwd)" || true
fi

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
    
    CARGO_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
    
    if [ -n "$VERSION_TAG" ]; then
        if [ "$CARGO_VERSION" != "$VERSION_TAG" ]; then
            echo "❌ Error: Version Mismatch!"
            echo "   Cargo.toml: $CARGO_VERSION"
            echo "   Git Tag:    $VERSION_TAG"
            return 1
        fi
    fi

    # 1. Sync check: Cargo.toml vs docs/myst.yml
    MYST_VERSION=$(grep 'logo_text: Kroki-rs V' docs/myst.yml | sed 's/.*Kroki-rs V//')
    if [ "$CARGO_VERSION" != "$MYST_VERSION" ]; then
        echo "❌ Error: Documentation Version Mismatch!"
        echo "   Cargo.toml:    $CARGO_VERSION"
        echo "   docs/myst.yml: $MYST_VERSION"
        return 1
    fi

    # 2. Sync check: Cargo.toml vs CHANGELOG.md
    if ! grep -q "## \[$CARGO_VERSION\]" CHANGELOG.md; then
        echo "❌ Error: CHANGELOG.md header for [$CARGO_VERSION] not found."
        return 1
    fi

    # 3. Sync check: Cargo.toml vs ROADMAP.md
    # (Simple check for existence of version header)
    if ! grep -i -q "v$CARGO_VERSION" ROADMAP.md; then
        echo "⚠️  Warning: ROADMAP.md might not mention v$CARGO_VERSION."
    fi

    # 4. Branch check (for release branches)
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$CURRENT_BRANCH" =~ ^v[0-9] ]] || [[ "$CURRENT_BRANCH" =~ ^release/v[0-9] ]]; then
        BRANCH_VER=${CURRENT_BRANCH#release/}
        BRANCH_VER=${BRANCH_VER#v}
        if [ "$CARGO_VERSION" != "$BRANCH_VER" ]; then
            echo "❌ Error: Branch Version Mismatch!"
            echo "   Cargo.toml: $CARGO_VERSION"
            echo "   Branch:     $CURRENT_BRANCH"
            return 1
        fi
    fi

    echo "✅ Version synchronization verified: v$CARGO_VERSION (Cargo, Docs, Changelog, Branch)"
    return 0
}

# Global variables for delta-based stats (Host only)
HOST_HITS_START=0
HOST_MISSES_START=0

manage_sccache() {
    local phase="$1"  # "before" or "after"
    local env_label="Host"
    [ "$IS_CONTAINER" = "true" ] && env_label="Container"
    
    # Only show stats if sccache is available
    if command -v sccache >/dev/null 2>&1; then
        if [ "$phase" = "before" ]; then
            # Record base stats for the Host to provide an accurate delta
            if [ "$IS_CONTAINER" != "true" ]; then
                local stats=$(sccache --show-stats 2>/dev/null)
                HOST_HITS_START=$(echo "$stats" | grep "Cache hits " | awk '{print $NF}' | head -n 1 || echo 0)
                HOST_MISSES_START=$(echo "$stats" | grep "Cache misses " | awk '{print $NF}' | head -n 1 || echo 0)
            fi
            
            if [ "$DEBUG_LOG" = "true" ]; then
                echo "📊 [SCCACHE] ($env_label) $phase run stats:"
                sccache --show-stats || true
            fi
        elif [ "$phase" = "after" ]; then
            if [ "$DEBUG_LOG" = "true" ]; then
                echo "📊 [SCCACHE] ($env_label) $phase run stats:"
                sccache --show-stats || true
            else
                # One-line summary for non-debug mode
                local stats=$(sccache --show-stats 2>/dev/null)
                if [ -n "$stats" ]; then
                    local hits=$(echo "$stats" | grep "Cache hits " | awk '{print $NF}' | head -n 1 || echo 0)
                    local misses=$(echo "$stats" | grep "Cache misses " | awk '{print $NF}' | head -n 1 || echo 0)
                    
                    # Calculate delta for Host
                    if [ "$IS_CONTAINER" != "true" ]; then
                        hits=$((hits - HOST_HITS_START))
                        misses=$((misses - HOST_MISSES_START))
                    fi

                    if [ "$hits" -ge 0 ] && [ "$misses" -ge 0 ]; then
                        echo "⚡️ [SCCACHE] ($env_label) Summary: $hits hits, $misses misses"
                    else
                        echo "⚡️ [SCCACHE] ($env_label) Summary: Active"
                    fi
                fi
            fi
        fi
    fi

    # Health check and recovery (only before run on host, as container is ephemeral)
    if [ "$phase" = "before" ] && [ "$IS_CONTAINER" != "true" ]; then
        if command -v sccache >/dev/null 2>&1; then
            echo "🔍 [SCCACHE] (Host) Verifying daemon health..."
            if ! sccache --show-stats >/dev/null 2>&1; then
                echo "⚠️  [SCCACHE] (Host) Daemon unresponsive. Attempting restart..."
                sccache --stop-server >/dev/null 2>&1 || true
                sccache --start-server >/dev/null 2>&1 || echo "❌ [SCCACHE] Failed to start server."
            fi

            # Fix permissions for the cache directory if it exists
            local cache_dir="$REPO_ROOT/.cargo-cache/sccache"
            if [ -d "$cache_dir" ]; then
                if [ "$(uname)" = "Darwin" ]; then
                    chmod -R 755 "$cache_dir" || true
                fi
            fi
        fi
    fi
}

# --- Initial Health Check (Execute immediately at script start) ---
manage_sccache "before"

# Mode: only version check (for GHA release workflow)
if [ "${1:-}" = "--version-check" ]; then
    run_version_check
    RET=$?
    manage_sccache "after"
    exit $RET
fi

# Mode: interactive shell in CI container (same mounts as ci-verify for fast incremental fixes)
if [ "${1:-}" = "--shell" ]; then
    export DOCKER_BUILDKIT=1
    DOCKER_CMD=$(make -s print-container-engine)
    [ -z "$DOCKER_CMD" ] && { echo "❌ Error: Podman or Docker not found."; exit 1; }
    CI_FINGERPRINT=$(make -s print-base-fingerprint)
    CI_IMAGE_LOCAL=$(make -s print-ci-image-local)
    CI_IMAGE_REMOTE=$(make -s print-ci-image-remote)
    echo "📦 Ensuring CI image is ready (fingerprint: ${CI_FINGERPRINT})..."
    if $DOCKER_CMD image inspect "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" >/dev/null 2>&1; then
        echo "✅ Found CI image locally (fingerprint: ${CI_FINGERPRINT})."
        $DOCKER_CMD tag "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" "$CI_IMAGE_LOCAL"
    elif $DOCKER_CMD pull "$CI_IMAGE_REMOTE" 2>/dev/null; then
        echo "✅ Pulled CI image from ghcr.io."
        $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}"
        $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL:latest"
        $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL"
    else
        echo "📦 CI image not in registry; building locally..."
        RUST_VER=$(make -s print-rust-version)
        $DOCKER_CMD build --target ci --build-arg RUST_VERSION="$RUST_VER" -t "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" -t "$CI_IMAGE_LOCAL" .
    fi
    echo "🐚 Opening shell in CI environment. Repo at /app (mounted); target at /app/target-ci (volume)."
    echo "   Cargo registry/git/sccache cached at .cargo-cache/ for fast incremental builds."
    echo "   Run 'make ghrun' or 'cargo test' for incremental fixes. Exit with 'exit'."
    mkdir -p "$(pwd)/target/ci" "$(pwd)/.cargo-cache/registry" "$(pwd)/.cargo-cache/git" "$(pwd)/.cargo-cache/sccache"
    
    $DOCKER_CMD run --rm -it \
        -v "$(pwd):/app" \
        -v "$(pwd)/target/ci:/app/target-ci" \
        -v "$(pwd)/.cargo-cache/registry:/usr/local/cargo/registry" \
        -v "$(pwd)/.cargo-cache/git:/usr/local/cargo/git" \
        -v "$(pwd)/.cargo-cache/sccache:/root/.cache/sccache" \
        -w /app \
        -e IS_CONTAINER=true \
        -e CARGO_TARGET_DIR=/app/target-ci \
        -e SCCACHE_DIR=/root/.cache/sccache \
        -e RUSTC_WRAPPER=sccache \
        -e SCCACHE_GHA_ENABLED=false \
        -e FEATURES="${FEATURES:-native-browser}" \
        -e PURGE_DISK \
        -e DEBUG_LOG \
        -e VERBOSE \
        -e NO_NETWORK \
        -e LOAD_TEST \
        -e SECURITY_TEST \
        ${JOBS:+-e JOBS=$JOBS} \
        --security-opt seccomp=unconfined \
        "$CI_IMAGE_LOCAL" \
        bash

    manage_sccache "after"
    exit 0
fi

# --- Normal ci-verify flow: version check upfront (unless in GHA to avoid redundancy), then container repro ---
if [ "$GITHUB_ACTIONS" != "true" ]; then
    run_version_check || true
fi

CLEANUP=${CLEANUP:-false}
export DOCKER_BUILDKIT=1

# Capture target if provided (default to ghrun)
TARGET="ghrun"
if [[ $# -gt 0 && ! "$1" =~ ^-- ]]; then
    TARGET="$1"
    shift
fi

# Fast-path: If already inside the CI container, execute directly.
# This MUST happen after TARGET is captured from arguments.
if [ "$IS_CONTAINER" = "true" ]; then
    echo "✅ Already inside CI container. Executing $TARGET directly..."
    # Any remaining arguments are passed to make (JOBS passed only if explicitly set)
    make $TARGET ${JOBS:+JOBS=$JOBS} "$@"
    manage_sccache "after"
    exit 0
fi

DOCKER_CMD=$(make -s print-container-engine)
if [ -z "$DOCKER_CMD" ]; then
    echo "❌ Error: Podman or Docker not found."
    exit 1
fi

echo "🚀 Using container engine: $DOCKER_CMD"
if ! $DOCKER_CMD system info >/dev/null 2>&1; then
    echo "❌ Error: Container engine ($DOCKER_CMD) is not responsive."
    echo "   Ensure Podman/Docker is running. If using Podman, you might need to run:"
    echo "   ./dflow setup"
    exit 1
fi

# Fingerprint (must match src-scripts/develop/vars/vars.mk and base-image.yml)
CI_FINGERPRINT=$(make -s print-base-fingerprint)
CI_IMAGE_LOCAL=$(make -s print-ci-image-local)
CI_IMAGE_REMOTE=$(make -s print-ci-image-remote)

echo "📦 Ensuring CI environment image is ready (fingerprint: ${CI_FINGERPRINT})..."
if $DOCKER_CMD image inspect "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" >/dev/null 2>&1; then
    echo "✅ Found CI image locally (fingerprint: ${CI_FINGERPRINT})."
    $DOCKER_CMD tag "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}" "$CI_IMAGE_LOCAL"
elif $DOCKER_CMD pull "$CI_IMAGE_REMOTE"; then
    echo "✅ Pulled CI image from ghcr.io (fingerprint: ${CI_FINGERPRINT})."
    $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "${CI_IMAGE_LOCAL}:${CI_FINGERPRINT}"
    $DOCKER_CMD tag "$CI_IMAGE_REMOTE" "$CI_IMAGE_LOCAL"
else
    echo "❌ Error: Failed to pull CI image from GHCR."
    echo "   Registry: ${CI_IMAGE_REMOTE}"
    echo "   Fingerprint: ${CI_FINGERPRINT}"
    echo "   Reason: Dockerfile has changed. To ensure portability, images must be generated on GitHub."
    echo "   Action: Wait for GHA 'Base Image' workflow to complete on main/release branch."
    exit 1
fi

# --- Toolchain Verification Guard ---
EXPECTED_RUST=$(grep '^channel =' rust-toolchain.toml | cut -d '"' -f 2)
echo "🔍 Verifying toolchain alignment (expected: ${EXPECTED_RUST})..."
ACTUAL_RUST=$($DOCKER_CMD run --rm "$CI_IMAGE_LOCAL" rustc --version | awk '{print $2}')
if [ "$ACTUAL_RUST" != "$EXPECTED_RUST" ]; then
    echo "❌ Error: Toolchain Mismatch detected!"
    echo "   Project requires: $EXPECTED_RUST"
    echo "   CI Image baked with: $ACTUAL_RUST"
    echo "   Please update RUST_VERSION in Dockerfile to $EXPECTED_RUST and trigger a base image build."
    exit 1
fi
echo "✅ Toolchain alignment verified."

echo "🧪 Running CI target '$TARGET' inside container..."
mkdir -p "$(pwd)/target/ci" "$(pwd)/.cargo-cache/registry" "$(pwd)/.cargo-cache/git" "$(pwd)/.cargo-cache/sccache"

$DOCKER_CMD run --rm \
    -v "$(pwd):/app" \
    -v "$(pwd)/target/ci:/app/target-ci" \
    -v "$(pwd)/.cargo-cache/registry:/usr/local/cargo/registry" \
    -v "$(pwd)/.cargo-cache/git:/usr/local/cargo/git" \
    -v "$(pwd)/.cargo-cache/sccache:/root/.cache/sccache" \
    -w /app \
    -e IS_CONTAINER=true \
    -e CARGO_TARGET_DIR=/app/target-ci \
    -e SCCACHE_DIR=/root/.cache/sccache \
    -e RUSTC_WRAPPER=sccache \
    -e SCCACHE_GHA_ENABLED=false \
    -e PURGE_DISK \
    -e DEBUG_LOG \
    -e VERBOSE \
    -e NO_NETWORK \
    -e LOAD_TEST \
    -e SECURITY_TEST \
    ${JOBS:+-e JOBS=$JOBS} \
    --security-opt seccomp=unconfined \
    "$CI_IMAGE_LOCAL" \
    bash src-scripts/ci-verify/repro-ci.sh $TARGET ${JOBS:+JOBS=$JOBS} "$@"

manage_sccache "after"

if [ "$PURGE_DISK" = "true" ]; then
    echo "🧹 Cleaning up images and builder cache..."
    $DOCKER_CMD system prune -f
    # Clean standard target directories
    rm -rf "$(pwd)/target" "$(pwd)/target/ci" "$(pwd)/.cargo-cache"
    # Clean stray directories and logs
    rm -rf "$(pwd)/target-ci" "$(pwd)/_build"
    rm -f "$(pwd)"/*.log
fi

echo "✅ Local CI verification (ci-verify) completed successfully."
