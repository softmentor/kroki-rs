#!/usr/bin/env bash
# src-scripts/gh-tasks/trigger-base-build.sh
# Purpose: Manually trigger the base image build only if the fingerprint is missing from GHCR.

set -e

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

# --- Configuration ---
WORKFLOW="base-image.yml"
DOCKER_ORG="softmentor"
IMAGE_BASE="kroki-rs-base"
IMAGE_CI="kroki-rs-ci"

# --- Safety Checks ---
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ Error: GitHub CLI ('gh') is not installed."
    exit 1
fi

# Determine container engine
DOCKER_CMD=$(make -s print-container-engine)
if [ -z "$DOCKER_CMD" ]; then
    echo "❌ Error: No container engine (podman/docker) found."
    exit 1
fi

# Calculate fingerprint
FINGERPRINT=$(make -s print-base-fingerprint)
echo "🔍 Local Fingerprint: $FINGERPRINT"

# --- Existence Check ---
echo "📡 Checking GHCR for existing image..."
IMAGE_URL="ghcr.io/${DOCKER_ORG}/${IMAGE_BASE}:${FINGERPRINT}"

# We use manifest inspect to check remote existence without pulling
if $DOCKER_CMD manifest inspect "$IMAGE_URL" >/dev/null 2>&1; then
    echo "✅ Image already exists on GHCR: $IMAGE_URL"
    echo "🚀 No build required. Local environments will pull this automatically."
    exit 0
fi

echo "⚠️  Image NOT found on GHCR: $IMAGE_URL"

# --- Trigger Workflow ---
echo "📤 Dispatching GitHub Action: $WORKFLOW..."
gh workflow run "$WORKFLOW" --ref "$(git rev-parse --abbrev-ref HEAD)"

echo "⏳ Workflow dispatched. You can track progress with:"
echo "   gh run list --workflow '$WORKFLOW' --limit 5"
echo ""
echo "🎉 Triggering process completed!"
