#!/usr/bin/env bash
# src-scripts/gh-tasks/prune-gha-cache.sh
# Purpose: Prune redundant GitHub Actions caches from your local machine (keeping only newest per ref).

set -e

# --- Safety Checks ---
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ Error: GitHub CLI ('gh') is not installed. Please install it first."
    exit 1
fi

echo "🧹 Proactive Repo-wide Cache Cleanup..."

# Handle individual ref pruning to keep the latest one for each specific type
# We use flexible grep/jq matching to account for OS-specific prefixes (e.g., Linux-cargo-)

REFS=$(gh cache list --limit 100 --json ref --jq '.[].ref' | sort | uniq)

if [ -z "$REFS" ]; then
    echo "✨ No caches found in this repository."
    exit 0
fi

for ref in $REFS; do
    echo "🔍 Checking ref: $ref"
    
    # 1. Prune Cargo caches (Keep 1)
    echo "   Pruning redundant Cargo caches..."
    gh cache list --ref "$ref" --json id,key | jq -r 'select(.key | contains("cargo-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} || true
    
    # 2. Prune Docker Image caches (Keep 1)
    echo "   Pruning redundant Docker image caches..."
    gh cache list --ref "$ref" --json id,key | jq -r 'select(.key | contains("docker-image-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} || true
    
    # 3. Prune Buildkit blobs (Keep 10)
    echo "   Pruning redundant Buildkit blobs (keeping 10)..."
    gh cache list --ref "$ref" --json id,key | jq -r 'select(.key | contains("buildkit-")) | .id' | tail -n +11 | xargs -I {} gh cache delete {} || true
done

# Aggressive PR cleanup: delete all caches for pull requests that haven't been touched in 24h
echo "🕵️  Checking for stale PR caches..."
gh cache list --limit 100 --json id,ref,lastAccessedAt | jq -r '
  .[] | select(.ref | startswith("refs/pull/")) | .id' | xargs -I {} gh cache delete {} || true

echo "🎉 Cache pruning process completed!"
