#!/bin/bash
# deploy-test.sh — Cross-compile pitop and deploy to a Raspberry Pi for testing
# Usage: ./scripts/deploy-test.sh [user@host] [target]
#
# Examples:
#   ./scripts/deploy-test.sh pi@raspberrypi            # auto-detect target
#   ./scripts/deploy-test.sh pi@pi5.local aarch64       # force 64-bit
#   ./scripts/deploy-test.sh pi@zero2w.local armv7       # force 32-bit

set -euo pipefail

HOST="${1:-pi@raspberrypi}"
TARGET_ARCH="${2:-aarch64}"

if [ "$TARGET_ARCH" = "aarch64" ]; then
    RUST_TARGET="aarch64-unknown-linux-gnu"
elif [ "$TARGET_ARCH" = "armv7" ]; then
    RUST_TARGET="armv7-unknown-linux-gnueabihf"
else
    echo "Unknown target: $TARGET_ARCH (use 'aarch64' or 'armv7')"
    exit 1
fi

echo "==> Building pitop for $RUST_TARGET..."
cargo build --release --target "$RUST_TARGET"

BINARY="target/$RUST_TARGET/release/pitop"

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    echo "Make sure you have the cross-compilation toolchain installed."
    echo "  rustup target add $RUST_TARGET"
    exit 1
fi

SIZE=$(du -h "$BINARY" | cut -f1)
echo "==> Binary size: $SIZE"

echo "==> Deploying to $HOST..."
scp "$BINARY" "$HOST:~/pitop"

echo "==> Running on $HOST..."
ssh -t "$HOST" "chmod +x ~/pitop && ~/pitop"
