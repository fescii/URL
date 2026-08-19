#!/usr/bin/env bash
set -e

REPO="fescii/URL"
BINARY="urls"

echo "==> Detecting OS and architecture..."
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    TARGET_OS="linux"
    ;;
  darwin)
    TARGET_OS="macos"
    ;;
  *)
    echo "Error: Unsupported operating system '$OS'."
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)
    TARGET_ARCH="x86_64"
    ;;
  arm64|aarch64)
    TARGET_ARCH="arm64"
    ;;
  *)
    echo "Error: Unsupported architecture '$ARCH'."
    exit 1
    ;;
esac

ASSET_NAME="urls-${TARGET_OS}-${TARGET_ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "==> Downloading ${BINARY} for ${TARGET_OS}-${TARGET_ARCH}..."
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_DIR/$ASSET_NAME" "$DOWNLOAD_URL"
else
  echo "Error: Neither curl nor wget found. Please install one to continue."
  exit 1
fi

echo "==> Extracting archive..."
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  if [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin"; then
    INSTALL_DIR="$HOME/.local/bin"
  fi
fi

echo "==> Installing ${BINARY} to ${INSTALL_DIR}..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
else
  sudo mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
fi

chmod +x "$INSTALL_DIR/$BINARY"

echo "==> Successfully installed ${BINARY} to ${INSTALL_DIR}/${BINARY}!"
if ! command -v "$BINARY" >/dev/null 2>&1; then
  echo "Note: Make sure ${INSTALL_DIR} is in your PATH."
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo "==> Try running:"
echo "  urls --help"
