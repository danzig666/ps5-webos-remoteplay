# webOS PS5 Remote M0

A Rust-first **hardware feasibility prototype**, scoped to M0 of the supplied plan. It plays a bundled, generated 1080p60 H.264 Annex-B fixture through the pinned Aurora SS4S NDL webOS5 backend, keeps an alpha EGL/SDL overlay visible, and diagnoses local controllers. **C2/C3 physical results are pending; M0 acceptance is not complete.**

No PS5 connection, Sony login, console changes, PC streaming service, pairing, audio playback, HEVC selection, adaptive triggers or M1 implementation is included. A PC is needed only to build/install/collect evidence. The installed prototype reads its fixture locally.

## Build

Linux x86_64, Python 3.11+ with pip, curl, git, patch, make, gcc, node/npm, readelf/ar are prerequisites. About 2 GB free space is needed. Install the workspace-local pinned tools:

```bash
./scripts/install-tools.sh
export M0_TOOLS_DIR="$PWD/.tools"
export CARGO_HOME="$M0_TOOLS_DIR/cargo"
export RUSTUP_HOME="$M0_TOOLS_DIR/rustup"
export WEBOS_SDK_DIR="$M0_TOOLS_DIR/arm-webos-linux-gnueabi_sdk-buildroot"
export PATH="$CARGO_HOME/bin:$M0_TOOLS_DIR/python/cmake/data/bin:$M0_TOOLS_DIR/node/node_modules/.bin:$PATH"
# Optional writable CLI data location (useful in restricted environments):
export APPDATA="$M0_TOOLS_DIR/cli-data"
python3 scripts/fetch-upstream.py
./scripts/build-deps.sh
./scripts/build-webos.sh
./scripts/package-ipk.sh
./scripts/verify.sh
```

`tools.lock.json` pins the SDK archive and Opus SHA-256, Rust, CMake and CLI. `upstream.lock.json` pins full Git revisions. Build commands refuse dirty/mismatched upstream inputs. Cargo has no third-party crates and builds offline with a lockfile. After fetching, only tool installation and dependency archive acquisition require network access. CLI transitive npm dependencies are captured in scripts/cli-package-lock.json (copy it with scripts/cli-package.json into a tool prefix and use `npm ci` when reproducing that tool).

For a same-path clean rebuild comparison after the first build, run `./scripts/repro-check.sh`. It verifies identical native-library and IPK checksums.

The output is `dist/org.homebrew.ps5remote.m0_0.1.0_arm.ipk`. Rust produces an ARMv7 soft-float static library; minimal C bootstrap/platform adapters own SDL/NDL calls on the same thread. All SS4S source is unchanged. The narrow source selection in `cmake/SS4SM0.cmake` avoids unrelated legacy backends. Opus is linked only because the existing NDL backend shares its audio/video implementation; the app opens video only. SDK NDL compatibility headers are supplied by the upstream SS4S fallback. TV NDL/EGL libraries remain on the TV. SDL3/webOS, SDL2 compatibility and Aurora’s NDL shim are packaged together. SDK SDL is dummy-only and is used for headers and the static test-font helper, never as the runtime renderer. libgcc_s and libatomic come from the pinned SDK.

The supplied fixture is 120 frames/2 seconds, H.264 High level 4.2, 1920×1080 at 60 fps, no B frames, AUD-delimited access units and repeated SPS/PPS at each IDR. Its exact file hash is in evidence. See `scripts/generate-fixture.sh` for regeneration; different FFmpeg/x264 versions may change its bytes, so normal builds use the checked-in fixture.

The app runs for 120 seconds, looping the clip. Rust bounds the fixture to 32 MiB, paces one access unit per frame deadline and skips to an IDR after feed errors/stalls; it never accumulates a catch-up queue. Error counts are submitted/feed statistics, **not decoded/presented FPS**. The prototype stops after a platform-open error, with no repeated NDL initialization attempts. A full decoder recovery design belongs after the feasibility gate.

## On the TV

Use standard Developer Mode / WebOS Dev Manager, without root or permission changes. Configure your own device alias via `ares-setup-device`/`ares-device --system-info` as supported by your CLI installation, then:

```bash
ares-install -d C2 dist/org.homebrew.ps5remote.m0_0.1.0_arm.ipk
ares-launch -d C2 org.homebrew.ps5remote.m0
./scripts/collect-tv.sh C2 evidence/hardware/C2-run-01
```

Repeat independently on C3. Actual commands require your TV alias/credentials; none are configured here. The log is `/tmp/ps5remote-m0.log`, replaced per launch. Collect it before another launch or reboot. It contains input values and device identities but no account keys or console data. Only the app tile should be launched for the normal sandbox test; an SSH-launched process may have different permissions/foreground state.

The overlay shows four controller slots, SDL button bits, raw normalized axes (0–5), advertised rumble, and the last rumble API result. START/Options or keyboard R/remote red requests a 500 ms test on attached pads. Keyboard G/remote green probes EVIOCGRAB **only if `M0_GRAB_DEVICE=/dev/input/eventN` was explicitly supplied** for a diagnostic launch. No automatic exclusive grabs occur. The probe acquires and immediately releases; closing its fd releases even if explicit release fails. Use the bus logs (USB=0x0003, Bluetooth=0x0005), controller name/GUID/path and physical transport together. Unknown transport stays unknown. Raw hidraw access is checked without sending any reports.

ESC/remote Back exits; SIGTERM also stops. Normal exit stops rumble, closes controllers, video and SDL. Startup and periodic scans cover already-paired pads and missed Bluetooth add events; instance-ID checks prevent duplicate opens. Unmapped devices require reviewing mappings, not evdev remapping guesses.

Follow [the hardware checklist](docs/hardware-checklist.md) and [run record](docs/run-template.md). [The report](docs/M0-report.md) records the actual build status and the review gate. No physical TV capability is confirmed by the host tests.
