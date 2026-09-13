#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# Runs as root before the payload is unpacked. Refuses to install on a
# macOS release older than the deployment target declared in the README.

set -u

VERSION=$(sw_vers -productVersion | cut -d. -f1)
if [[ "$VERSION" -lt 12 ]]; then
    echo "Error: macOS 12.0 or later required. Current: $(sw_vers -productVersion)"
    exit 1
fi
exit 0
