#!/usr/bin/env bash
# src-scripts/gh-tasks/gh-check-wrapper.sh
# Purpose: Wrap a command and report its outcome as a GitHub Check Run using curl.
# Usage: ./gh-check-wrapper.sh <check_name> <command...>

set -e

CHECK_NAME="$1"
shift
COMMAND="$@"

if [ -z "$GITHUB_TOKEN" ] || [ -z "$GITHUB_SHA" ] || [ -z "$GITHUB_REPOSITORY" ]; then
    echo "ℹ️  Not in GitHub Actions or missing tokens. Executing command directly without reporting..."
    eval "$COMMAND"
    exit $?
fi

API_URL="https://api.github.com/repos/${GITHUB_REPOSITORY}/check-runs"
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# 1. Create Check Run (Status: in_progress)
echo "🚀 Creating GitHub Check: $CHECK_NAME"
RESPONSE=$(curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "$API_URL" \
  -d "{\"name\":\"$CHECK_NAME\",\"head_sha\":\"$GITHUB_SHA\",\"status\":\"in_progress\",\"started_at\":\"$STARTED_AT\"}")

CHECK_ID=$(echo "$RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)

if [ -z "$CHECK_ID" ]; then
    echo "⚠️  Failed to create Check Run (API Response: $RESPONSE). Executing command anyway..."
    eval "$COMMAND"
    exit $?
fi

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
COMPLETED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
curl -s -X PATCH \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "$API_URL/$CHECK_ID" \
  -d "{\"status\":\"completed\",\"conclusion\":\"$CONCLUSION\",\"completed_at\":\"$COMPLETED_AT\"}" \
  > /dev/null

exit $EXIT_CODE
