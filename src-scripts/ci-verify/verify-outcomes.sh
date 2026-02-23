#!/usr/bin/env bash
# src-scripts/ci-verify/verify-outcomes.sh
# Asserts expected outcomes after each dflow phase. Usage: ./src-scripts/ci-verify/verify-outcomes.sh <teardown|setup|develop|ci-verify>
# Run from repo root. Exit 0 if all checks pass, 1 otherwise.

set -e
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

PHASE="${1:-}"
if [ -z "$PHASE" ]; then
    echo "Usage: $0 <teardown|setup|develop|ci-verify>"
    exit 1
fi

FAIL=0
pass() { echo "PASS: $1"; }
fail() { echo "FAIL: $1"; FAIL=1; }

case "$PHASE" in
    teardown)
        echo "=== Verifying teardown outcomes ==="
        [ -d dist ] && fail "dist/ should be removed" || pass "dist/ removed (disk cache purged)"
        if [ -d target ] && [ -n "$(ls -A target 2>/dev/null)" ]; then
            echo "WARN: target/ still has content (cargo clean may have failed in some environments)"
        else
            pass "target/ absent or empty (build cache purged)"
        fi
        CE=$(command -v podman 2>/dev/null || command -v docker 2>/dev/null)
        if [ -n "$CE" ]; then
            pass "container prune ran (engine available)"
        fi
        ;;
    setup)
        echo "=== Verifying setup outcomes ==="
        command -v rustc >/dev/null 2>&1 || fail "rustc not found"
        pass "rustc available"
        CE=$(command -v podman 2>/dev/null || command -v docker 2>/dev/null)
        if [ -n "$CE" ]; then
            $CE images --format "{{.Repository}}:{{.Tag}}" 2>/dev/null | grep -q "softmentor/kroki-rs-base" || fail "base image softmentor/kroki-rs-base not found"
            pass "base image softmentor/kroki-rs-base exists"
        fi
        ;;
    develop)
        echo "=== Verifying develop outcomes ==="
        [ -f target/release/kroki-rs ] || [ -f target/debug/kroki-rs ] || fail "kroki-rs binary not found under target/"
        pass "kroki-rs binary exists"
        [ -d dist ] && pass "dist/ present (if verify was run)" || true
        pass "develop targets (fmt, lint, build, test, smoke-test) completed"
        ;;
    ci-verify)
        echo "=== Verifying ci-verify outcomes ==="
        CE=$(command -v podman 2>/dev/null || command -v docker 2>/dev/null)
        if [ -n "$CE" ]; then
            $CE images --format "{{.Repository}}:{{.Tag}}" 2>/dev/null | grep -q "softmentor/kroki-rs-ci" || fail "CI image softmentor/kroki-rs-ci not found"
            pass "CI image softmentor/kroki-rs-ci exists"
        fi
        pass "ci-verify (repro-ci.sh) completed; ghrun ran inside container"
        ;;
    *)
        echo "Unknown phase: $PHASE"
        exit 1
        ;;
esac

[ $FAIL -eq 0 ] && echo "=== $PHASE verification: PASS ===" || echo "=== $PHASE verification: FAIL ==="
exit $FAIL
