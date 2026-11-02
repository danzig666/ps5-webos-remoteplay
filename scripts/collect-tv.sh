#!/usr/bin/env bash
# Read-only log collection. Installation/launch is intentionally a separate operator step.
set -euo pipefail
: "${1:?Usage: collect-tv.sh DEVICE OUTPUT_DIR}"
: "${2:?Usage: collect-tv.sh DEVICE OUTPUT_DIR}"
mkdir -p "$2"
ares-device --system-info -d "$1" > "$2/device-info.txt" 2>&1
ares-pull -d "$1" /tmp/ps5remote-m0.log "$2/ps5remote-m0.log"
printf 'Record model, firmware, privilege mode, controller transport and physical observations in docs/run-template.md\n'
