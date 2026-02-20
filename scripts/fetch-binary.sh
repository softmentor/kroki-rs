#!/bin/bash
set -e

# fetch-binary.sh: Downloads a pre-built Linux binary for fast Docker packaging.
# Usage: ./scripts/fetch-binary.sh [version] [arch]

VERSION=${1:-$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)}
ARCH=${2:-"arm64"} # Default to arm64 since user is on Mac M1/M2/M3

BINARY_NAME="kroki-rs"
REPO="softmentor/kroki-rs"
DIST_DIR="dist"

echo "Fetching ${BINARY_NAME} v${VERSION} for linux/${ARCH}..."

mkdir -p ${DIST_DIR}

# Note: This assumes a release exists on GitHub with the standard naming convention.
# If no release exists, this will fail gracefully.
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_NAME}-linux-${ARCH}"

if curl --output /dev/null --silent --head --fail "$URL"; then
    echo "Downloading from $URL..."
    curl -L "$URL" -o "${DIST_DIR}/${BINARY_NAME}"
    chmod +x "${DIST_DIR}/${BINARY_NAME}"
    echo "✅ Binary downloaded to ${DIST_DIR}/${BINARY_NAME}"
else
    echo "❌ Binary v${VERSION} not found at GitHub Releases."
    echo "You might need to run 'make release' locally (if on Linux) or wait for CI to publish the tag."
    exit 1
fi
