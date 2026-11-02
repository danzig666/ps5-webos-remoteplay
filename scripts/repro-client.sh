#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
for run in before after; do
  cmake --build build/client --target clean
  cargo clean -p ps5remote-app --release --target armv7-unknown-linux-gnueabi
  scripts/build-client.sh > "evidence/client-repro-$run-build.txt" 2>&1
  scripts/package-client.sh > "evidence/client-repro-$run-package.txt" 2>&1
  sha256sum build/client-stage/ps5remote build/client-stage/lib/*.so* dist/org.homebrew.ps5remote_0.2.0_arm.ipk > "evidence/client-repro-$run.txt"
done
diff -u evidence/client-repro-before.txt evidence/client-repro-after.txt
printf 'Identical client, native libraries and IPK after clean rebuilds\n'
