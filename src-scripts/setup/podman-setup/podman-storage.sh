#!/bin/bash
set -e

# src-scripts/setup/podman-setup/podman-storage.sh
# Purpose: Relocate Podman machine storage to an external volume to save host disk space.
# Usage: export PODMAN_STORAGE_DIR="/Volumes/your-external-drive/podman" && ./src-scripts/setup/podman-setup/podman-storage.sh

if [ -z "$PODMAN_STORAGE_DIR" ]; then
    echo "ℹ️ PODMAN_STORAGE_DIR is not set. Using default local storage (~/.local/share/containers)."
    exit 0
fi

echo "🚀 Podman Storage Redirector"
echo "Target: $PODMAN_STORAGE_DIR"

mkdir -p "$PODMAN_STORAGE_DIR"

SOURCE_DIR="$HOME/.local/share/containers/podman/machine"

if [ -d "$SOURCE_DIR" ] && [ ! -L "$SOURCE_DIR" ]; then
    echo "⚠️ Existing Podman machine data found at $SOURCE_DIR"
    echo "Moving existing data to $PODMAN_STORAGE_DIR..."
    
    podman machine stop || true
    
    if command -v rsync >/dev/null 2>&1; then
        rsync -a --exclude="*.sock" "$SOURCE_DIR/" "$PODMAN_STORAGE_DIR/machine/"
    else
        cp -R "$SOURCE_DIR/" "$PODMAN_STORAGE_DIR/machine/"
    fi
    
    if [ -d "$PODMAN_STORAGE_DIR/machine" ]; then
        rm -rf "$SOURCE_DIR"
        ln -s "$PODMAN_STORAGE_DIR/machine" "$SOURCE_DIR"
        echo "✅ Podman machine storage successfully relocated."
    else
        echo "❌ Error: Failed to copy data to $PODMAN_STORAGE_DIR."
        exit 1
    fi
elif [ -L "$SOURCE_DIR" ]; then
    echo "✅ Podman machine storage is already symlinked to $(readlink "$SOURCE_DIR")"
else
    echo "📦 Initializing Podman storage symlink..."
    mkdir -p "$(dirname "$SOURCE_DIR")"
    ln -s "$PODMAN_STORAGE_DIR" "$SOURCE_DIR"
    echo "✅ Symlink created."
fi

echo "ℹ️ Note: If you have performance issues, ensure the external volume is formatted with APFS or ExFAT (with caution)."
