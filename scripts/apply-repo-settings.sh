#!/bin/bash
set -e

# Configuration
ORG="softmentor"
REPO="kroki-rs"
TEMPLATE="scripts/repo-settings.json"

echo "--- Processing $ORG/$REPO ---"

if ! command -v gh &> /dev/null; then
    echo "❌ Error: 'gh' CLI not found. Please install it first."
    exit 1
fi

if [ ! -f "$TEMPLATE" ]; then
    echo "❌ Error: Template file $TEMPLATE not found."
    exit 1
fi

# 1. Update General Repository Settings
echo "Updating general settings..."
jq '.repository' "$TEMPLATE" | gh api --method PATCH "/repos/$ORG/$REPO" --input - --silent

# 2. Update Branch Protection (on 'main')
echo "Applying branch protection to 'main'..."
# Protection requires a specific structure for 'required_status_checks'
jq '.branch_protection' "$TEMPLATE" | gh api --method PUT "/repos/$ORG/$REPO/branches/main/protection" --input - --silent

echo "✅ Successfully configured $REPO according to Kroki-Flow protocol."
