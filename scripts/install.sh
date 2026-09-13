#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# ntfs-mac installer: installs macFUSE + ntfs-3g on macOS.
#
# Usage: install.sh [--yes] [--help]
#
# Shipped to /usr/local/share/ntfs-mac/install.sh by scripts/build-macos.sh,
# and also runnable straight from a source checkout.
set -euo pipefail

YES=false
HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --yes|-y) YES=true; shift ;;
        --help|-h) HELP=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if $HELP; then
    echo "ntfs-mac installer"
    echo ""
    echo "Usage: ${0##*/} [--yes] [--help]"
    echo ""
    echo "Options:"
    echo "  --yes, -y    Non-interactive mode (auto-confirm)"
    echo "  --help, -h   Show this help"
    exit 0
fi

if ! command -v brew &>/dev/null; then
    echo "Error: Homebrew not found. Install from https://brew.sh"
    exit 1
fi

ARCH=$(uname -m)
echo "Detected architecture: $ARCH"

if [[ "$ARCH" == "arm64" ]]; then
    FUSE_CASK="fuse-t"
    FUSE_DESC="FUSE-T (Apple Silicon native)"
else
    FUSE_CASK="macfuse"
    FUSE_DESC="macFUSE"
fi

if ! $YES; then
    echo ""
    echo "This script will install:"
    echo "  1. $FUSE_DESC ($FUSE_CASK)"
    echo "  2. ntfs-3g"
    echo ""
    read -rp "Continue? [y/N] " response
    case "$response" in
        [yY][eE][sS]|[yY]) ;;
        *) echo "Aborted."; exit 0 ;;
    esac
fi

echo ""
echo "==> Installing $FUSE_DESC..."
if brew list --cask "$FUSE_CASK" &>/dev/null; then
    echo "    $FUSE_CASK already installed"
else
    brew install --cask "$FUSE_CASK"
    echo "    NOTE: You may need to approve the kernel extension in:"
    echo "    System Settings > Privacy & Security"
    echo "    You may need to restart after installation."
fi

echo ""
echo "==> Installing ntfs-3g..."
if brew list ntfs-3g &>/dev/null; then
    echo "    ntfs-3g already installed"
else
    brew install ntfs-3g
fi

echo ""
echo "==> Verifying installation..."
MISSING=0
for tool in ntfs-3g newfs_ntfs ntfsfix fsck_ntfs; do
    if command -v "$tool" &>/dev/null; then
        echo "  ✓ $tool"
    else
        echo "  ✗ $tool (missing)"
        MISSING=$((MISSING + 1))
    fi
done

if [[ $MISSING -gt 0 ]]; then
    echo ""
    echo "Warning: $MISSING tools missing. You may need to restart your terminal."
fi

echo ""
echo "==> Done!"
echo "  Run 'ntfs-mac doctor' to verify all dependencies."
echo "  Run 'ntfs-mac list' to see your NTFS volumes."
