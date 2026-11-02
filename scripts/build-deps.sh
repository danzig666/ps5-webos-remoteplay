#!/usr/bin/env bash
set -euo pipefail
export SOURCE_DATE_EPOCH=1793577600
cd "$(dirname "$0")/.."
: "${WEBOS_SDK_DIR:?Set WEBOS_SDK_DIR}"
python3 scripts/download-locked.py opus build/downloads/opus-1.5.2.tar.gz
mkdir -p build/deps
if [[ ! -d build/deps/opus-1.5.2 ]]; then tar -xzf build/downloads/opus-1.5.2.tar.gz -C build/deps; fi
cmake -S build/deps/opus-1.5.2 -B build/opus -DCMAKE_TOOLCHAIN_FILE="$WEBOS_SDK_DIR/share/buildroot/toolchainfile.cmake" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DOPUS_BUILD_TESTING=OFF -DOPUS_BUILD_PROGRAMS=OFF
cmake --build build/opus --parallel 2
DESTDIR="$PWD/build/deps-root" cmake --install build/opus
# Aurora's production SDL3 + SDL2 compatibility path; SDK SDL is dummy-only.
python3 scripts/check-pins.py
python3 scripts/download-locked.py sdl3_webos build/downloads/SDL3-webos.tar.gz
mkdir -p build/deps/sdl3-webos
tar -xzf build/downloads/SDL3-webos.tar.gz -C build/deps/sdl3-webos
if [[ ! -f build/deps/sdl2-compat/.m0-patched ]]; then
  mkdir -p build/deps/sdl2-compat
  if [[ -e upstream/sdl2-compat/.git ]]; then
    git -C upstream/sdl2-compat archive HEAD | tar -x -C build/deps/sdl2-compat
  else
    cp -R upstream/sdl2-compat/. build/deps/sdl2-compat/
  fi
  (cd build/deps/sdl2-compat && patch -p1 -i ../../../upstream/aurora-tv/scripts/webos/sdl2-compat-webos-scancode.patch && patch -p1 -i ../../../upstream/aurora-tv/scripts/webos/sdl2-compat-private-soname.patch)
  touch build/deps/sdl2-compat/.m0-patched
fi
cmake -S build/deps/sdl2-compat -B build/sdl2-compat \
  -DCMAKE_TOOLCHAIN_FILE="$WEBOS_SDK_DIR/share/buildroot/toolchainfile.cmake" \
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$PWD/build/sdl-runtime" -DCMAKE_INSTALL_LIBDIR=lib \
  -DSDL2COMPAT_TESTS=OFF -DSDL2COMPAT_INSTALL=ON -DSDL2COMPAT_INSTALL_CPACK=OFF \
  -DSDL2COMPAT_STATIC=OFF -DSDL2COMPAT_WERROR=OFF -DSDL2COMPAT_X11=OFF \
  -DSDL2COMPAT_VENDOR_INFO="webOS SDL3" -DSDL3_INCLUDE_DIRS="$PWD/build/deps/sdl3-webos/include"
cmake --build build/sdl2-compat --parallel 2
cmake --install build/sdl2-compat
cp -L build/deps/sdl3-webos/lib/libSDL3.so.0.4.16 build/sdl-runtime/lib/libSDL3.so.0
