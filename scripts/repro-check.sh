#!/usr/bin/env bash
# Compare two clean object rebuilds at the same tool/source paths.
set -euo pipefail
cd "$(dirname "$0")/.."
for run in before after; do
  cmake --build build/webos --target clean
  cmake --build build/sdl2-compat --target clean
  cargo clean --release --target armv7-unknown-linux-gnueabi
  scripts/build-deps.sh > "evidence/repro-$run-deps.txt" 2>&1
  scripts/build-webos.sh > "evidence/repro-$run-build.txt" 2>&1
  scripts/package-ipk.sh > "evidence/repro-$run-package.txt" 2>&1
  sha256sum build/stage/m0 build/stage/lib/*.so* dist/*.ipk > "evidence/repro-$run.txt"
done
diff -u evidence/repro-before.txt evidence/repro-after.txt
printf 'Identical clean-build native and IPK checksums\n'
