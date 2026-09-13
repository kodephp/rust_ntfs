#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# Runs as root after the payload has been unpacked.
#
# The payload already places every file at its final location, so this
# script only has to fix permissions. The previous version symlinked
# /usr/local/bin/ntfs-mac to a file inside ntfs-mac.app that the payload
# never created, which produced a dangling symlink.

set -u

BUNDLE_ID="com.kodephp.ntfs-mac"
CLI="/usr/local/bin/ntfs-mac"
SHARE="/usr/local/share/ntfs-mac"

if [[ -f "$CLI" ]]; then
    chmod 755 "$CLI" 2>/dev/null || true
fi

if [[ -d "$SHARE" ]]; then
    chmod 755 "$SHARE" 2>/dev/null || true
    chmod 644 "$SHARE/LICENSE" "$SHARE/NOTICE" \
              "$SHARE/THIRD_PARTY_LICENSES.md" "$SHARE/README.md" 2>/dev/null || true
    chmod 755 "$SHARE/install.sh" 2>/dev/null || true
fi

# Releases 0.1.0–0.1.2 built their payload at the filesystem root, so an
# upgrade would otherwise leave a second copy of the app in `/`. The bundle
# is only removed after confirming it is ours; generic names such as
# /LICENSE and /README.md are deliberately left untouched.
if [[ -f "/ntfs-mac.app/Contents/Info.plist" ]] \
   && grep -q "$BUNDLE_ID" "/ntfs-mac.app/Contents/Info.plist" 2>/dev/null; then
    rm -rf "/ntfs-mac.app" 2>/dev/null || true
fi

exit 0
