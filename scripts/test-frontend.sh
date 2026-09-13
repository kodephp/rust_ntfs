#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# Frontend quality gate for the ntfs-mac GUI (crates/ntfs-mac-tauri/src).
#
# Checks JS syntax, then runs the frontend/backend contract test that fails if
# the window's i18n dictionary drifts from the menu-bar `Labels` in lib.rs.
#
# Usage: scripts/test-frontend.sh

set -euo pipefail

cd "$(dirname "$0")/.."
SRC=crates/ntfs-mac-tauri/src

command -v node >/dev/null 2>&1 || { echo "error: node is required" >&2; exit 1; }

echo "== JS syntax =="
node --check "$SRC/app.js"
echo "  ✓ node --check passed"

echo
echo "== contract =="
node scripts/test-frontend.mjs
