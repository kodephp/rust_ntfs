#!/bin/bash
if [[ -f "$pkginfo/ntfs-mac" ]]; then
    ln -sf "/Applications/ntfs-mac.app/Contents/Resources/ntfs-mac" "/usr/local/bin/ntfs-mac" 2>/dev/null || true
    chmod +x "/Applications/ntfs-mac.app/Contents/Resources/ntfs-mac" 2>/dev/null || true
fi
chmod +x "/Applications/ntfs-mac.app/Contents/Resources/scripts/install.sh" 2>/dev/null || true
exit 0
