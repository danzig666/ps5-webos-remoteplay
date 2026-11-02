#!/usr/bin/env bash
# Host-only memory/undefined-behavior checks; no TV or gamepad access.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p build/sanitizers
headers="$PWD/build/sdl-runtime/include/SDL2"
for test in navigation hid-report controller-selection; do
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
