#!/usr/bin/env bash
set -euo pipefail

# Install ACE from GitHub releases.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/prod9/ace/main/install.sh | bash
#
# Installs the latest release binary to /usr/local/bin/ace.

REPO="prod9/ace"
INSTALL_DIR="/usr/local/bin"

# --- Detect platform ----------------------------------------------------------

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) TRIPLE_OS="apple-darwin" ;;
  Linux)  TRIPLE_OS="unknown-linux-gnu" ;;
  *)
    echo "Error: unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  aarch64|arm64) TRIPLE_ARCH="aarch64" ;;
  x86_64)        TRIPLE_ARCH="x86_64" ;;
  *)
    echo "Error: unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="${TRIPLE_ARCH}-${TRIPLE_OS}"

# --- Resolve latest release ---------------------------------------------------

echo "Fetching latest release..."
RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
TAG="$(curl -fsSL "$RELEASE_URL" | grep '"tag_name"' | sed 's/.*: "\(.*\)".*/\1/')"

if [ -z "$TAG" ]; then
  echo "Error: could not determine latest release tag."
  exit 1
fi

# --- Download binary ----------------------------------------------------------

ASSET_URL="https://github.com/${REPO}/releases/download/${TAG}/ace-${TARGET}"
TMPFILE="$(mktemp)"
trap 'rm -f "$TMPFILE"' EXIT

echo "Downloading ace ${TAG} (${TARGET})..."
curl -fsSL -o "$TMPFILE" "$ASSET_URL"

if [ ! -s "$TMPFILE" ]; then
  echo "Error: download failed or produced empty file."
  exit 1
fi

# --- Install ------------------------------------------------------------------

chmod +x "$TMPFILE"
mkdir -p "$INSTALL_DIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPFILE" "${INSTALL_DIR}/ace"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "$TMPFILE" "${INSTALL_DIR}/ace"
fi

echo "Installed ace ${TAG} to ${INSTALL_DIR}/ace"
