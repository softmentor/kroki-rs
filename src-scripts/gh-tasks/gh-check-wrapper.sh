#!/usr/bin/env bash
# src-scripts/gh-tasks/gh-check-wrapper.sh
# Purpose: Wrap a command and report its outcome as a GitHub Check Run.
# Usage: ./gh-check-wrapper.sh <check_name> <command...>

set -e

CHECK_NAME="$1"
shift
COMMAND="$@"

if [ -z "$GITHUB_TOKEN" ] || [ -z "$GITHUB_SHA" ]; then
    echo "ℹ️  Not in GitHub Actions or missing tokens. Executing command directly without reporting..."
    eval "$COMMAND"
    exit $?
fi

# 1. Create Check Run (Status: in_progress)
echo "🚀 Creating GitHub Check: $CHECK_NAME"
CHECK_ID=$(gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/${GITHUB_REPOSITORY}/check-runs \
  -f name="$CHECK_NAME" \
  -f head_sha="$GITHUB_SHA" \
  -f status="in_progress" \
  -f started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --jq '.id')

# 2. Execute Command
echo "🏃 Running: $COMMAND"
set +e
eval "$COMMAND"
EXIT_CODE=$?
set -e

# 3. Determine Conclusion
CONCLUSION="success"
if [ $EXIT_CODE -ne 0 ]; then
    CONCLUSION="failure"
fi

# 4. Finalize Check Run
echo "🏁 Finalizing GitHub Check: $CHECK_NAME (Conclusion: $CONCLUSION)"
gh api \
  --method PATCH \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/${GITHUB_REPOSITORY}/check-runs/$CHECK_ID \
  -f status="completed" \
  -f conclusion="$CONCLUSION" \
  -f completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --silent

exit $EXIT_CODE
