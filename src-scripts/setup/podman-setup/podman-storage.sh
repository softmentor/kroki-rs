#!/bin/bash
set -e

# src-scripts/setup/podman-setup/podman-storage.sh
# Purpose: Relocate large Podman VM disk images (.raw) to an external volume
#          to save host disk space, while keeping sockets and metadata local.
#
# Why file-level symlinks:
#   ExFAT/FAT32 volumes do not support Unix sockets. Podman creates .sock files
#   inside the machine directory tree, so symlinking the entire directory to an
#   external volume breaks `podman machine start`. Instead, we symlink only the
#   heavy .raw disk images (typically 100 GB) and leave everything else local.
#
# Usage: export PODMAN_STORAGE_DIR="/Volumes/your-external-drive/podman" && \
#        ./src-scripts/setup/podman-setup/podman-storage.sh
#
# Must run AFTER `podman machine init` so that .raw files exist to relocate.

if [ -z "$PODMAN_STORAGE_DIR" ]; then
    echo "ℹ️  PODMAN_STORAGE_DIR is not set. Using default local storage."
    exit 0
fi

echo "🚀 Podman Storage Redirector (file-level)"
echo "   Target volume: $PODMAN_STORAGE_DIR"

mkdir -p "$PODMAN_STORAGE_DIR"

MACHINE_DIR="$HOME/.local/share/containers/podman/machine"
MOVED=0

# Find all .raw VM disk images under the machine directory
for RAW_FILE in $(find "$MACHINE_DIR" -name "*.raw" -type f 2>/dev/null); do
    REL_PATH="${RAW_FILE#$MACHINE_DIR/}"
    DEST_DIR="$PODMAN_STORAGE_DIR/$(dirname "$REL_PATH")"
    DEST_FILE="$DEST_DIR/$(basename "$RAW_FILE")"
    SIZE=$(du -sh "$RAW_FILE" 2>/dev/null | cut -f1)

    echo "   Moving $REL_PATH ($SIZE) → $DEST_DIR/"
    mkdir -p "$DEST_DIR"
    mv "$RAW_FILE" "$DEST_FILE"
    ln -s "$DEST_FILE" "$RAW_FILE"
    MOVED=$((MOVED + 1))
done

if [ "$MOVED" -gt 0 ]; then
    echo "✅ Relocated $MOVED disk image(s) to $PODMAN_STORAGE_DIR"
else
    # Check if already symlinked from a previous run
    LINKED=$(find "$MACHINE_DIR" -name "*.raw" -type l 2>/dev/null | wc -l | tr -d ' ')
    if [ "$LINKED" -gt 0 ]; then
        echo "✅ $LINKED disk image(s) already on external storage"
    else
        echo "ℹ️  No .raw disk images found. Run this after: podman machine init"
    fi
fi
