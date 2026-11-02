#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
command -v ares-package >/dev/null || { echo 'Install @webos-tools/cli 3.2.0 (ares-package)' >&2; exit 2; }
python3 scripts/audit-stage.py
mkdir -p dist
SOURCE_DATE_EPOCH=1793577600 ares-package build/stage -o dist
python3 scripts/canonical-ipk.py dist/*.ipk
sha256sum dist/*.ipk
