#!/bin/sh
# pitop installer — https://pitop.hongjunwu.com/install.sh
#
# Usage:
#   curl -sL https://pitop.hongjunwu.com/install.sh | sh
#
# Or pin a version:
#   curl -sL https://pitop.hongjunwu.com/install.sh | PITOP_VERSION=v0.1.0 sh
#
# Supported platforms:
#   - Raspberry Pi 5 / 4B / Zero 2W (64-bit OS, aarch64)
#   - Raspberry Pi 4B / 3B+ (32-bit OS, armv7l)

set -eu

REPO="wu-hongjun/pitop"
VERSION="${PITOP_VERSION:-latest}"

# --- Detect architecture ---
ARCH=$(uname -m)
case "$ARCH" in
    aarch64)    ASSET_ARCH="aarch64" ;;
    armv7l)     ASSET_ARCH="armv7" ;;
    armv6l)
        echo "Error: ARMv6 (Pi Zero v1 / Pi 1) is not supported."
        echo "If you have a Pi Zero 2W, install a 64-bit OS for aarch64 support."
        exit 1 ;;
    x86_64)     ASSET_ARCH="x86_64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        echo "pitop supports aarch64, armv7l, and x86_64."
        exit 1 ;;
esac

# --- Detect OS ---
OS=$(uname -s)
if [ "$OS" != "Linux" ]; then
    echo "Error: pitop only supports Linux (detected: $OS)"
    exit 1
fi

# --- Resolve version ---
if [ "$VERSION" = "latest" ]; then
    echo "Fetching latest version..."
    VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' | head -1 | cut -d'"' -f4)
fi

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine latest version."
    echo "Check https://github.com/$REPO/releases for available versions."
    echo "Or pin a version: curl -sL ... | PITOP_VERSION=v0.1.0 sh"
    exit 1
fi

case "$VERSION" in
    v[0-9]*) ;;
    *) echo "Error: Invalid version format: $VERSION (expected vX.Y.Z)"; exit 1 ;;
esac

# --- Download ---
ASSET="pitop-${VERSION}-${ASSET_ARCH}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading pitop $VERSION for $ARCH..."
HTTP_CODE=$(curl -sL -w "%{http_code}" -o "$TMPDIR/pitop.tar.gz" "$URL")

if [ "$HTTP_CODE" != "200" ]; then
    echo "Error: Download failed (HTTP $HTTP_CODE)"
    echo "URL: $URL"
    echo ""
    echo "This could mean:"
    echo "  - Version $VERSION does not exist"
    echo "  - No binary is available for $ASSET_ARCH"
    echo "  - GitHub is temporarily unreachable"
    echo ""
    echo "Try building from source instead:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  cargo install --git https://github.com/$REPO"
    exit 1
fi

# --- Extract ---
tar xzf "$TMPDIR/pitop.tar.gz" -C "$TMPDIR" 2>/dev/null || {
    echo "Error: Failed to extract archive. The download may be corrupt."
    exit 1
}

# Find the binary (might be in a subdirectory)
BINARY=$(find "$TMPDIR" -name "pitop" -type f -perm -001 2>/dev/null | head -1)
if [ -z "$BINARY" ]; then
    BINARY=$(find "$TMPDIR" -name "pitop" -type f 2>/dev/null | head -1)
fi

if [ -z "$BINARY" ]; then
    echo "Error: Could not find pitop binary in the archive."
    exit 1
fi

# --- Install ---
INSTALL_DIR="/usr/local/bin"
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY" "$INSTALL_DIR/pitop"
    chmod +x "$INSTALL_DIR/pitop"
elif command -v sudo >/dev/null 2>&1; then
    echo "Installing to $INSTALL_DIR (requires sudo)..."
    sudo mv "$BINARY" "$INSTALL_DIR/pitop"
    sudo chmod +x "$INSTALL_DIR/pitop"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    mv "$BINARY" "$INSTALL_DIR/pitop"
    chmod +x "$INSTALL_DIR/pitop"

    # Check if ~/.local/bin is in PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            echo ""
            echo "Note: $INSTALL_DIR is not in your PATH."
            echo "Add it with:  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo "Then run:     source ~/.bashrc"
            ;;
    esac
fi

echo ""
echo "pitop $VERSION installed to $INSTALL_DIR/pitop"
echo "Run 'pitop' to start the system monitor."
