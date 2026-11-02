#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${WEBOS_SDK_DIR:?Set WEBOS_SDK_DIR to the relocated buildroot-nc4 2026.08-webos SDK}"
for tool in cargo cmake python3 make; do command -v "$tool" >/dev/null || { echo "Missing $tool" >&2; exit 2; }; done
python3 scripts/check-pins.py

export SOURCE_DATE_EPOCH=1793577600
export CARGO_INCREMENTAL=0
cargo build -p webos-m0 --release --locked --offline --target armv7-unknown-linux-gnueabi
cmake -S . -B build/webos -DCMAKE_BUILD_TYPE=Release -DCMAKE_TOOLCHAIN_FILE="$WEBOS_SDK_DIR/share/buildroot/toolchainfile.cmake" -DCMAKE_INSTALL_LIBDIR=lib
cmake --build build/webos --parallel 2
cmake --install build/webos --prefix "$PWD/build/stage"
cp -R packaging/. build/stage/
# Depend on pinned SDK copies; never copy TV-owned NDL/EGL/libc libraries.
cp -L build/sdl-runtime/lib/libSDL2-webos.so.0 build/stage/lib/
cp -L build/sdl-runtime/lib/libSDL3.so.0 build/stage/lib/
cp LICENSE THIRD_PARTY.md upstream.lock.json build/stage/
cp -L "$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/lib/libgcc_s.so.1" build/stage/lib/
cp -L "$WEBOS_SDK_DIR/arm-webos-linux-gnueabi/sysroot/lib/libatomic.so.1" build/stage/lib/
python3 scripts/audit-stage.py
