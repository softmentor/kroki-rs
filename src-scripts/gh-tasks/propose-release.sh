#!/usr/bin/env bash
# src-scripts/gh-tasks/propose-release.sh
# Purpose: Automate pushing the release branch and creating a PR to main.

set -e

# --- Configuration ---
MAIN_BRANCH="main"

# --- Safety Checks ---
echo "🔍 Performing pre-proposal checks..."

# 1. Check if gh CLI is installed
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ Error: GitHub CLI ('gh') is not installed. Please install it first."
    exit 1
fi

# 2. Check current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ ! "$CURRENT_BRANCH" =~ ^release/v.* ]] && [[ ! "$CURRENT_BRANCH" =~ ^rel/v.* ]] && [[ ! "$CURRENT_BRANCH" =~ ^feat/.* ]]; then
    echo "⚠️  Warning: Current branch '$CURRENT_BRANCH' does not follow release/vX.Y.Z or feat/* naming."
    read -p "🤔 Do you want to proceed with this branch? (y/N): " confirm_branch
    if [[ ! "$confirm_branch" =~ ^[Yy]$ ]]; then
        echo "🚫 Proposal aborted."
        exit 0
    fi
fi

# 3. Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: You have uncommitted changes. Please commit or stash them before proposing."
    exit 1
fi

# 4. Verify local state with ci-verify (Recommended)
echo "🧪 Reminder: Ensure you have passed './dflow ci-verify' before proposing."

# --- Execution ---

# 1. Push branch
echo "📤 Pushing $CURRENT_BRANCH to origin..."
git push origin "$CURRENT_BRANCH"

# 2. Create PR
echo "📝 Creating Pull Request to $MAIN_BRANCH..."

PR_TITLE="release: finalize $CURRENT_BRANCH"
if [[ "$CURRENT_BRANCH" =~ ^feat/.* ]]; then
    PR_TITLE="feat: ${CURRENT_BRANCH#feat/}"
fi

# Try to create the PR
gh pr create \
    --base "$MAIN_BRANCH" \
    --head "$CURRENT_BRANCH" \
    --title "$PR_TITLE" \
    --body "Automated release proposal from $CURRENT_BRANCH. Includes final cleanup and verification." || \
echo "⚠️  PR might already exist or creation failed. Check GitHub UI."

echo "🎉 Proposal process completed for $CURRENT_BRANCH!"
