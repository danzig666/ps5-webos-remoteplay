#!/usr/bin/env bash
set -euo pipefail
: "${1:?Usage: collect-client.sh DEVICE OUTPUT_DIR}"
: "${2:?Usage: collect-client.sh DEVICE OUTPUT_DIR}"
mkdir -p "$2"
ares-device --system-info -d "$1" > "$2/device-info.txt" 2>&1
ares-pull -d "$1" /tmp/ps5remote.log "$2/ps5remote.log"
printf 'Complete docs/client-hardware-tests.md and attach observations; never include pairing.json or account IDs.\n'
