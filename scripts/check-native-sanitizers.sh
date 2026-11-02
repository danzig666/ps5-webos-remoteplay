#!/usr/bin/env bash
# Host-only memory/undefined-behavior checks; no TV or gamepad access.
set -euo pipefail
cd "$(dirname "$0")/.."
: "${WEBOS_SDK_DIR:?Set WEBOS_SDK_DIR for SDL headers}"
mkdir -p build/sanitizers
headers="$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/usr/include/SDL2"
for test in hid-report controller-selection; do
  gcc -std=c11 -Wall -Wextra -ffunction-sections -fdata-sections -Wl,--gc-sections \
    -fsanitize=address,undefined -fno-omit-frame-pointer \
    -I"$headers" -Iplatform/client/vendor -Iplatform/client/compat \
    -Iupstream/aurora-tv/core/moonlight-common-c/src \
    "tests/$test-test.c" -lm -o "build/sanitizers/$test"
  "build/sanitizers/$test"
done
gcc -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -fno-omit-frame-pointer platform/webos/device_probe.c tests/device-probe-test.c \
  -o build/sanitizers/device-probe
build/sanitizers/device-probe
printf 'Host C address/undefined-behavior checks passed (no hardware access)\n'
