#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${WEBOS_SDK_DIR:?Set the relocated checksum-pinned webOS SDK directory}"
export SOURCE_DATE_EPOCH=1793577600 CARGO_INCREMENTAL=0
python3 scripts/check-pins.py
python3 scripts/check-extraction.py
cargo build -p ps5remote-app --release --locked --offline --target armv7-unknown-linux-gnueabi
# The preserved M0 target shares the platform library build graph.
cargo build -p webos-m0 --release --locked --offline --target armv7-unknown-linux-gnueabi
cmake -S . -B build/client -DBUILD_CLIENT=ON -DCMAKE_BUILD_TYPE=Release -DCMAKE_TOOLCHAIN_FILE="$WEBOS_SDK_DIR/share/buildroot/toolchainfile.cmake" -DCMAKE_INSTALL_LIBDIR=lib
cmake --build build/client --parallel 2
mkdir -p build/client-stage
cmake --install build/client --prefix "$PWD/build/client-stage" --component Unspecified
cmake --install build/client --prefix "$PWD/build/client-stage" --component Client
mkdir -p build/client-stage/assets
cp packaging/assets/gamecontrollerdb.txt packaging/assets/gamecontrollerdb_extra.txt build/client-stage/assets/
cp -R packaging/licenses build/client-stage/
rm -f build/client-stage/assets/test.h264
cp packaging/client/appinfo.json packaging/icon.png build/client-stage/
cp -L build/sdl-runtime/lib/libSDL2-webos.so.0 build/sdl-runtime/lib/libSDL3.so.0 build/client-stage/lib/
cp -L "$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/lib/libgcc_s.so.1" "$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/lib/libatomic.so.1" build/client-stage/lib/
cp packaging/licenses/AGPL-3.0.txt build/client-stage/LICENSE
cp THIRD_PARTY.md upstream.lock.json tools.lock.json build/client-stage/
python3 scripts/audit-client.py
