#!/bin/sh
set -e

# Kroki-rs Installer
# Usage: curl -sSfL https://raw.githubusercontent.com/softmentor/kroki-rs/main/install.sh | sh

GITHUB_REPO="softmentor/kroki-rs"
BINARY_NAME="kroki-rs"
INSTALL_DIR="/usr/local/bin"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
  linux*)  PLATFORM="linux" ;;
  darwin*) PLATFORM="darwin" ;;
  *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "🚀 Installing $BINARY_NAME for $OS ($ARCH)..."

# Fetch latest release version from GitHub
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
  echo "Error: Could not fetch latest release from GitHub."
  exit 1
fi

# Construct download URL (assuming the release assets follow the format: kroki-rs-darwin.tar.gz or kroki-rs-linux.tar.gz)
# Note: In a real scenario, we might have arch-specific names, but based on Makefile, it's just platform-based for now.
# However, to be future-proof, let's stick to the Makefile's logic for now or adjust it.
# Current Makefile generates: $(BINARY_NAME)-$(PLATFORM).tar.gz
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$LATEST_RELEASE/$BINARY_NAME-$PLATFORM.tar.gz"

echo "📥 Downloading $DOWNLOAD_URL..."
TMP_DIR=$(mktemp -d)
curl -L "$DOWNLOAD_URL" -o "$TMP_DIR/$BINARY_NAME.tar.gz"

echo "📦 Extracting..."
tar -xzf "$TMP_DIR/$BINARY_NAME.tar.gz" -C "$TMP_DIR"

echo "⚙️  Installing to $INSTALL_DIR (may require sudo)..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
  sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

rm -rf "$TMP_DIR"

echo "✅ $BINARY_NAME $LATEST_RELEASE installed successfully!"

# Dependency Check
echo "\n🔍 Checking dependencies..."
if command -v dot >/dev/null 2>&1; then
  echo "  - Graphviz: Found ($(dot -V 2>&1 | head -n 1))"
else
  echo "  - ⚠️  Graphviz not found. Install it for Graphviz/PlantUML support."
fi

if command -v node >/dev/null 2>&1; then
  echo "  - Node.js: Found ($(node -v))"
else
  echo "  - ⚠️  Node.js not found. Install it for Mermaid/Vega/Wavedrom support."
fi

echo "\n🎉 You are ready to go! Run '$BINARY_NAME serve' to start the service."
