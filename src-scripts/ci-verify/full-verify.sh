#!/usr/bin/env bash
# src-scripts/ci-verify/full-verify.sh
# Runs a full verification from a clean state: teardown → setup → develop → ci-verify,
# with outcome assertions after each phase. Exit 0 only if all phases and checks pass.
# Usage: ./src-scripts/ci-verify/full-verify.sh [optional: --skip-ci-verify]
# Run from repo root. --skip-ci-verify skips the long containerized ci-verify step.

set -e
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"
VERIFY="$REPO_ROOT/src-scripts/ci-verify/verify-outcomes.sh"
SKIP_CI_VERIFY=false
for arg in "$@"; do
    [ "$arg" = "--skip-ci-verify" ] && SKIP_CI_VERIFY=true
done

echo "=== Full verification (teardown → setup → develop → ci-verify) ==="
echo ""

echo ">>> Phase: teardown"
./dflow teardown
"$VERIFY" teardown
echo ""

echo ">>> Phase: setup"
./dflow setup
"$VERIFY" setup
echo ""

echo ">>> Phase: develop"
./dflow develop
"$VERIFY" develop
echo ""

if [ "$SKIP_CI_VERIFY" = true ]; then
    echo ">>> Phase: ci-verify (skipped by --skip-ci-verify)"
    echo "Run manually: ./dflow ci-verify && $VERIFY ci-verify"
else
    echo ">>> Phase: ci-verify (this may take 15+ minutes on first run)"
    ./dflow ci-verify
    "$VERIFY" ci-verify
fi

echo ""
echo "=== Full verification: PASS ==="
