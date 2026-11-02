#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo test --locked --offline
mkdir -p build
gcc -std=c11 -Wall -Wextra -Werror platform/webos/device_probe.c tests/device-probe-test.c -o build/device-probe-test
build/device-probe-test
python3 scripts/check-pins.py
python3 -m compileall -q scripts
for f in scripts/*.sh; do bash -n "$f"; done
ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=codec_name,width,height,r_frame_rate,has_b_frames,nb_read_frames -of json packaging/assets/test.h264
python3 scripts/audit-stage.py
python3 scripts/verify-ipk.py
