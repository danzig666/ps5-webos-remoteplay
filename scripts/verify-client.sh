#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${WEBOS_SDK_DIR:?Set WEBOS_SDK_DIR for the SDK SDL headers}"
cargo test --workspace --locked --offline
cargo fmt -p ps5remote-app -- --check
python3 scripts/check-pins.py
python3 scripts/check-extraction.py
mkdir -p build
sdk_headers="$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/usr/include/SDL2"
gcc -std=c11 -Wall -Wextra -ffunction-sections -fdata-sections -Wl,--gc-sections -I"$sdk_headers" -Iplatform/client/vendor -Iplatform/client/compat -Iupstream/aurora-tv/core/moonlight-common-c/src tests/hid-report-test.c -o build/hid-report-test
build/hid-report-test
printf 'USB HID byte/strength/reset capture passed (no hardware access)\n'
gcc -std=c11 -Wall -Wextra -Werror platform/webos/device_probe.c tests/device-probe-test.c -o build/device-probe-test
build/device-probe-test
for f in scripts/*.sh; do bash -n "$f"; done
python3 -m compileall -q scripts
python3 scripts/audit-client.py
python3 scripts/verify-client-ipk.py
