#!/usr/bin/env bash
# src-scripts/gh-tasks/prune-gha-cache.sh
# Purpose: Prune redundant GitHub Actions caches to keep them within limits.

set -e

# --- Safety Checks ---
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ Error: GitHub CLI ('gh') is not installed. Please install it first."
    exit 1
fi

echo "🧹 CI Cache Management..."

# 1. Stale PR cleanup: delete caches for pull requests not accessed in the last 24 hours.
# This is always done to prevent abandoned PRs from taking up space.
echo "🕵️  Checking for stale PR caches (>24h since last access)..."
gh cache list --limit 100 --json id,ref,lastAccessedAt | jq -r '
  .[] | select(.ref | startswith("refs/pull/"))
       | select((.lastAccessedAt | sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601) < (now - 86400))
       | .id' | xargs -I {} gh cache delete {} || true

# 2. Capacity-based Pruning: Only perform aggressive per-ref pruning if total size > 80% (8GB)
# This avoids unnecessary API calls and preserves cache whenever possible.
TOTAL_SIZE=$(gh cache list --limit 100 --json sizeInBytes --jq '[.[].sizeInBytes] | add // 0')
THRESHOLD=8589934592 # 8GB in bytes

echo "📊 Total Cache Size: $((TOTAL_SIZE / 1024 / 1024)) MB / 10240 MB"

if [ "$TOTAL_SIZE" -lt "$THRESHOLD" ]; then
    echo "✨ Cache size is within safe limits (80%). Skipping aggressive pruning."
    exit 0
fi

echo "⚠️  Cache limit threshold (80%) reached. Starting aggressive per-ref pruning..."

REFS=$(gh cache list --limit 100 --json ref --jq '.[].ref' | sort | uniq)

for ref in $REFS; do
    echo "🔍 Pruning ref: $ref"
    
    # Prune Cargo caches (Keep 1)
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("cargo-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} || true
    
    # Prune Docker image caches (Keep 1)
    # Note: Focuses on "docker-ci-" as the current standard.
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("docker-ci-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} || true
    
    # Prune Buildkit blobs (Keep 10)
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("buildkit-")) | .id' | tail -n +11 | xargs -I {} gh cache delete {} || true
done

echo "🎉 Cache pruning process completed!"
