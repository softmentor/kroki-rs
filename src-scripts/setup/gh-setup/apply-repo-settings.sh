#!/bin/bash
set -e

# src-scripts/setup/gh-setup/apply-repo-settings.sh
# Purpose: Apply GitHub repository and branch-protection settings from a JSON template.
# Usage: Run from repo root. Requires 'gh' CLI and admin access.
# Template: src-scripts/setup/gh-setup/repo-settings.json

ORG="softmentor"
REPO="kroki-rs"
TEMPLATE="src-scripts/setup/gh-setup/repo-settings.json"

echo "--- Processing $ORG/$REPO ---"

if ! command -v gh &> /dev/null; then
    echo "❌ Error: 'gh' CLI not found. Please install it first."
    exit 1
fi

if [ ! -f "$TEMPLATE" ]; then
    echo "❌ Error: Template file $TEMPLATE not found."
    exit 1
fi

echo "Updating general settings..."
jq '.repository' "$TEMPLATE" | gh api --method PATCH "/repos/$ORG/$REPO" --input - --silent

echo "Applying branch protection to 'main'..."
jq '.branch_protection' "$TEMPLATE" | gh api --method PUT "/repos/$ORG/$REPO/branches/main/protection" --input - --silent

echo "✅ Successfully configured $REPO according to Kroki-Flow protocol."
