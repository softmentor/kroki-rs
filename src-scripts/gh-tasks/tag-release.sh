#!/usr/bin/env bash
# src-scripts/gh-tasks/tag-release.sh
# Purpose: Professional release tagging utility with safety checks.

set -e

# --- Configuration ---
BINARY_NAME="kroki-rs"
MAIN_BRANCH="main"

# --- Safety Checks ---
echo "🔍 Performing pre-release checks..."

# 1. Check current branch
CURRENT_BRANCH=$(git rev-parse --abrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "$MAIN_BRANCH" ]; then
    echo "❌ Error: You must be on the '$MAIN_BRANCH' branch to tag a release."
    echo "   Current branch: $CURRENT_BRANCH"
    exit 1
fi

# 2. Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: You have uncommitted changes. Please commit or stash them before tagging."
    exit 1
fi

# 3. Check for untracked files
if [ -n "$(git ls-files --others --exclude-standard)" ]; then
    echo "⚠️  Warning: You have untracked files. Continuing, but ensure they are not required."
fi

# 4. Verify version in Cargo.toml
CARGO_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
echo "📦 Current version in Cargo.toml: v$CARGO_VERSION"

# 5. Build documentation sanity check
echo "📚 Verifying documentation build..."
cargo doc --no-deps > /dev/null 2>&1
echo "✅ Documentation build passed."

# --- Interaction ---
read -p "🚀 Do you want to tag and push v$CARGO_VERSION to remote? (y/N): " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "🚫 Release aborted."
    exit 0
fi

# --- Argument Parsing ---
DRY_RUN="false"
for arg in "$@"; do
    if [[ "$arg" == "--dry-run" ]]; then
        DRY_RUN="true"
    fi
done

# --- Execution ---
TAG="v$CARGO_VERSION"

if [[ "$DRY_RUN" == "true" ]]; then
    echo "🏗️  [DRY-RUN] Would tag: $TAG"
    echo "🏗️  [DRY-RUN] Would push tag $TAG to origin"
else
    echo "🏷️  Tagging $TAG..."
    git tag -a "$TAG" -m "Release $TAG"

    echo "📤 Pushing tags to origin..."
    git push origin "$TAG"
fi

echo "🎉 Release $TAG initiated! GHA 'release.yml' will handle distribution."
