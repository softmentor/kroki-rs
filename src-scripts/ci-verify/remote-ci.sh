#!/bin/bash
set -e

# src-scripts/ci-verify/remote-ci.sh (Repository-Driven Version)
# Purpose: Offload CI verification to a remote host by pulling directly from GitHub.
# Usage: REMOTE_HOST="user@remote-ip" [BRANCH="..."] [REPO_URL="..."] bash src-scripts/ci-verify/remote-ci.sh

if [ -z "$REMOTE_HOST" ]; then
    echo "❌ Error: REMOTE_HOST is not set."
    echo "Usage: REMOTE_HOST=\"user@remote-ip\" [BRANCH=\"feature/v0.0.5\"] [REPO_URL=\"...\"] bash src-scripts/ci-verify/remote-ci.sh"
    exit 1
fi

REPO_URL=${REPO_URL:-$(git config --get remote.origin.url)}
BRANCH=${BRANCH:-$(git rev-parse --abbrev-ref HEAD)}
REMOTE_DIR=${REMOTE_DIR:-"~/kroki-rs-remote-ci"}
MAX_LOGS=5

echo "🚀 Repository-Driven Remote CI (ci-verify)"
echo "Target Host: $REMOTE_HOST"
echo "Repository : $REPO_URL"
echo "Branch     : $BRANCH"
echo "Directory  : $REMOTE_DIR"

echo "🧪 Triggering verification suite (SSH Agent Forwarding enabled)..."
ssh -t -A "$REMOTE_HOST" \
    PURGE_DISK="$PURGE_DISK" \
    DEBUG_LOG="$DEBUG_LOG" \
    VERBOSE="$VERBOSE" \
    NO_NETWORK="$NO_NETWORK" \
    LOAD_TEST="$LOAD_TEST" \
    JOBS="$JOBS" \
    bash -s <<EOF
    set -e
    
    if ! command -v git &>/dev/null; then
        echo "❌ Error: git is not installed on the remote host."
        exit 1
    fi

    if [ ! -d "$REMOTE_DIR/.git" ]; then
        echo "📦 Initializing clean clone from $REPO_URL..."
        mkdir -p "$REMOTE_DIR"
        git clone "$REPO_URL" "$REMOTE_DIR"
    fi

    cd "$REMOTE_DIR"
    echo "🔄 Fetching and switching to $BRANCH..."
    git fetch origin
    git checkout "$BRANCH"
    git pull origin "$BRANCH"

    mkdir -p logs
    echo "🧹 Rotating logs (Max: $MAX_LOGS)..."
    ls -1t logs/remotecitest-*.log 2>/dev/null | tail -n +$MAX_LOGS | xargs rm -f || true

    TIMESTAMP=\$(date +%Y%m%d-%H%M%S)
    LOG_FILE="logs/remotecitest-\$TIMESTAMP.log"
    echo "📝 Execution started at \$TIMESTAMP. Log: \$LOG_FILE"
    
    bash src-scripts/ci-verify/repro-ci.sh 2>&1 | tee "\$LOG_FILE"
    
    echo "✅ Remote ci-verify complete."
EOF

echo "🏁 Remote flow finished."
