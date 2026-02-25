#!/usr/bin/env bash
# src-scripts/gh-tasks/prune-gha-runs.sh
# Purpose: Prune old or failed GitHub Actions workflow runs.

set -e

# --- Safety Checks ---
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ Error: GitHub CLI ('gh') is not installed. Please install it first."
    exit 1
fi

LIMIT=${1:-100}
DELETE_FAILED=${2:-true}

echo "🧹 Starting GitHub Actions workflow runs cleanup..."

# 1. Prune Failed and Canceled Runs
if [ "$DELETE_FAILED" = "true" ]; then
    echo "🔍 Identifying failed and canceled runs..."
    FAILED_RUNS=$(gh run list --status failure --limit 1000 --json databaseId --jq '.[].databaseId')
    CANCELED_RUNS=$(gh run list --status cancelled --limit 1000 --json databaseId --jq '.[].databaseId')
    
    BAD_RUNS="$FAILED_RUNS $CANCELED_RUNS"
    
    if [ -n "$(echo $BAD_RUNS | tr -d ' ')" ]; then
        COUNT=$(echo "$BAD_RUNS" | wc -w | tr -d ' ')
        echo "🗑️  Deleting $COUNT failed/canceled runs..."
        echo "$BAD_RUNS" | xargs -I {} gh run delete {}
    else
        echo "✨ No failed or canceled runs found."
    fi
fi

# 2. Keep only latest N runs
echo "🔍 Checking for runs to prune (keeping latest $LIMIT)..."
ALL_RUNS=$(gh run list --limit 1000 --json databaseId --jq '.[].databaseId')
TOTAL_COUNT=$(echo "$ALL_RUNS" | wc -w | tr -d ' ')

if [ "$TOTAL_COUNT" -gt "$LIMIT" ]; then
    TO_DELETE_COUNT=$((TOTAL_COUNT - LIMIT))
    echo "🗑️  Deleting $TO_DELETE_COUNT oldest runs to keep only the latest $LIMIT..."
    # Get IDs starting from LIMIT+1
    OLD_RUNS=$(echo "$ALL_RUNS" | tr ' ' '\n' | tail -n +$((LIMIT + 1)))
    echo "$OLD_RUNS" | xargs -I {} gh run delete {}
else
    echo "✨ Total runs ($TOTAL_COUNT) is within the limit ($LIMIT). No pruning needed."
fi

echo "🎉 Workflow runs cleanup completed!"
