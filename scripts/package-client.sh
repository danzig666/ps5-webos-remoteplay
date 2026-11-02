#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/audit-client.py
mkdir -p dist
SOURCE_DATE_EPOCH=1793577600 ares-package build/client-stage -o dist
python3 scripts/canonical-ipk.py dist/org.homebrew.ps5remote_0.2.0_arm.ipk
python3 scripts/verify-client-ipk.py
sha256sum dist/org.homebrew.ps5remote_0.2.0_arm.ipk
