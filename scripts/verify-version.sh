#!/bin/bash
set -e

# Support both local execution and GitHub Actions
REF=${GITHUB_REF:-$(git symbolic-ref -q HEAD || git describe --tags --always)}
VERSION_TAG=""

if [[ $REF == refs/tags/v* ]]; then
    VERSION_TAG=${REF#refs/tags/v}
elif [[ $REF == v* ]]; then
    VERSION_TAG=${REF#v}
fi

if [ -z "$VERSION_TAG" ]; then
    echo "ℹ️ Not a version tag ($REF). Skipping version synchronicity check."
    exit 0
fi

CARGO_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)

if [ "$CARGO_VERSION" != "$VERSION_TAG" ]; then
    echo "❌ Error: Version Mismatch!"
    echo "   Cargo.toml: $CARGO_VERSION"
    echo "   Git Tag:    $VERSION_TAG"
    echo "   Please update Cargo.toml to match the tag before releasing."
    exit 1
fi

echo "✅ Version synchronization verified: v$CARGO_VERSION"
