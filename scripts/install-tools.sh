#!/usr/bin/env bash
# Workspace-local tools. Does not change system packages or shell startup files.
set -euo pipefail
cd "$(dirname "$0")/.."
tools_dir="${M0_TOOLS_DIR:-$PWD/.tools}"
mkdir -p "$tools_dir"
export CARGO_HOME="$tools_dir/cargo" RUSTUP_HOME="$tools_dir/rustup"
curl -fL https://sh.rustup.rs -o "$tools_dir/rustup-init.sh"
sh "$tools_dir/rustup-init.sh" -y --no-modify-path --profile minimal --default-toolchain 1.99.1
"$CARGO_HOME/bin/rustup" target add --toolchain 1.99.1 armv7-unknown-linux-gnueabi
"$CARGO_HOME/bin/rustup" component add rustfmt --toolchain 1.99.1
python3 -m pip install --target "$tools_dir/python" cmake==3.31.6
mkdir -p "$tools_dir/node"
cp scripts/cli-package.json "$tools_dir/node/package.json"
cp scripts/cli-package-lock.json "$tools_dir/node/package-lock.json"
npm ci --cache "$tools_dir/npm-cache" --prefix "$tools_dir/node"
python3 scripts/download-locked.py sdk "$tools_dir/sdk.tar.gz"
tar -xzf "$tools_dir/sdk.tar.gz" -C "$tools_dir"
"$tools_dir/arm-webos-linux-gnueabi_sdk-buildroot/relocate-sdk.sh"
printf 'Tools installed in %s; see README.md for environment exports.\n' "$tools_dir"
