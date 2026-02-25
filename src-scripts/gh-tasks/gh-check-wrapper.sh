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

# Resilient parsing: search for "id": <number>
# handles spaces via xargs and matches accurately even if line formatting varies
CHECK_ID=$(echo "$RESPONSE" | grep -o '"id":\s*[0-9]*' | head -1 | cut -d: -f2 | xargs || true)

if [ -z "$CHECK_ID" ]; then
    echo "⚠️  Failed to extract Check ID from API Response."
    echo "Debug: API response was: $RESPONSE"
    echo "Executing command anyway..."
    eval "$COMMAND"
    exit $?
fi

echo "✅ Created Check ID: $CHECK_ID"

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
FINAL_RESPONSE=$(curl -s -X PATCH \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "$API_URL/$CHECK_ID" \
  -d "{\"status\":\"completed\",\"conclusion\":\"$CONCLUSION\",\"completed_at\":\"$COMPLETED_AT\"}")

# Log final response if it failed
if echo "$FINAL_RESPONSE" | grep -q '\"message\"'; then
    echo "⚠️  Failed to finalize check run: $FINAL_RESPONSE"
fi

exit $EXIT_CODE
