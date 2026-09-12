#!/bin/bash
VERSION=$(sw_vers -productVersion | cut -d. -f1)
if [[ "$VERSION" -lt 12 ]]; then
    echo "Error: macOS 12.0 or later required. Current: $(sw_vers -productVersion)"
    exit 1
fi
exit 0
