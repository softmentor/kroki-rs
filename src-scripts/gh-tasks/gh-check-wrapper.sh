#!/usr/bin/env bash
# src-scripts/gh-tasks/gh-check-wrapper.sh
# Purpose: Wrap a command and report its outcome as a GitHub Commit Status.
# Usage: ./gh-check-wrapper.sh <context_name> <command...>

set -e

CONTEXT_NAME="$1"
shift
COMMAND="$@"

# Detect the correct SHA to report on (Head SHA for PRs, usual SHA for Push)
REPORT_SHA="${GITHUB_HEAD_SHA:-$GITHUB_SHA}"

if [ -z "$GITHUB_TOKEN" ] || [ -z "$REPORT_SHA" ] || [ -z "$GITHUB_REPOSITORY" ]; then
    echo "ℹ️  Not in GitHub Actions or missing tokens. Executing command directly..."
    eval "$COMMAND"
    exit $?
fi

API_URL="https://api.github.com/repos/${GITHUB_REPOSITORY}/statuses/${REPORT_SHA}"
TARGET_URL="https://github.com/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"

# 1. Report Status: pending
echo "🚀 Reporting Status: pending ($CONTEXT_NAME) on $REPORT_SHA"
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "$API_URL" \
  -d "{\"state\":\"pending\",\"context\":\"$CONTEXT_NAME\",\"description\":\"Running $CONTEXT_NAME...\",\"target_url\":\"$TARGET_URL\"}" \
  > /dev/null

# 2. Execute Command
echo "🏃 Running: $COMMAND"
set +e
eval "$COMMAND"
EXIT_CODE=$?
set -e

# 3. Determine Final State
STATE="success"
DESCRIPTION="$CONTEXT_NAME passed"
if [ $EXIT_CODE -ne 0 ]; then
    STATE="failure"
    DESCRIPTION="$CONTEXT_NAME failed"
fi

# 4. Finalize Status
echo "🏁 Finalizing Status: $STATE ($CONTEXT_NAME) on $REPORT_SHA"
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "$API_URL" \
  -d "{\"state\":\"$STATE\",\"context\":\"$CONTEXT_NAME\",\"description\":\"$DESCRIPTION\",\"target_url\":\"$TARGET_URL\"}" \
  > /dev/null

exit $EXIT_CODE
