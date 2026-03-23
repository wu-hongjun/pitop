#!/bin/sh
set -eu

REPO="OWNER/pitop"
VERSION="${PITOP_VERSION:-latest}"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    aarch64)    ASSET_ARCH="aarch64" ;;
    armv7l)     ASSET_ARCH="armv7" ;;
    armv6l)     echo "ARMv6 (Pi Zero/Pi 1) is not supported. Use a 64-bit OS on Zero 2W."; exit 1 ;;
    *)          echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Resolve version
if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
fi

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine latest version"
    exit 1
fi

# Validate version format
case "$VERSION" in
    v[0-9]*) ;; # Looks like a version tag
    *) echo "Error: Invalid version string: $VERSION"; exit 1 ;;
esac

# Download
URL="https://github.com/$REPO/releases/download/$VERSION/pitop-$VERSION-$ASSET_ARCH.tar.gz"
TMPDIR=$(mktemp -d)
echo "Downloading pitop $VERSION for $ARCH..."
curl -sL "$URL" -o "$TMPDIR/pitop.tar.gz"

# Extract
tar xzf "$TMPDIR/pitop.tar.gz" -C "$TMPDIR"

# Install
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

mv "$TMPDIR/pitop" "$INSTALL_DIR/pitop"
chmod +x "$INSTALL_DIR/pitop"

# Cleanup
rm -rf "$TMPDIR"

echo "pitop $VERSION installed to $INSTALL_DIR/pitop"
echo "Run 'pitop' to start the system monitor"
