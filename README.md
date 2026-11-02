# Native Rust-first PS5 Remote Play for LG webOS

An ARMv7 native client for LG OLED C2/C3, built around pinned PS5Upload Remote Play protocol code and Aurora SDL3/SS4S/NDL platform code. Pairing and streaming connect directly to the PS5 over the LAN. No Sony login, desktop engine, HTTP/WebSocket relay, or always-on PC is included.

**Development build 0.2.0. M1–M6 software paths are implemented; real PS5 sessions and successful C2/C3 validation remain pending; a user-reported C3 launch failure is recorded below.** Hardware tests are deferred by the user's instruction, not treated as passed. This is not a verified release. See [implementation status and limits](docs/implementation-status.md).

Latest [software bug-check report](docs/bug-check-2026-11-02.md): seven findings fixed, 147 Rust tests passing, native sanitizer checks and reproducible ARM/IPK builds. The updated `v0.2.0-dev.2` IPK includes these fixes.

## Downloads

**Current C3 diagnostic build:** [Download dev.4 IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.4/artifacts/v0.2.0-dev.4/org.homebrew.ps5remote_0.2.0_arm.ipk) · [Input diagnostics and retest instructions](artifacts/v0.2.0-dev.4/README.md). Dev.3 displays the menu but the user reports physical arrows do nothing. Input freeze remains unresolved.

**C3 startup correction:** [Download dev.3 IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.3/artifacts/v0.2.0-dev.3/org.homebrew.ps5remote_0.2.0_arm.ipk) · [Cause, fixes and retest instructions](artifacts/v0.2.0-dev.3/README.md). The user-provided dev.2 log confirmed a storage-directory rejection before window initialization. Successful launch with dev.3 is awaiting hardware retest.

[Download the updated IPK (v0.2.0-dev.2)](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.2/artifacts/v0.2.0-dev.2/org.homebrew.ps5remote_0.2.0_arm.ipk) · [Checksum and build details](artifacts/v0.2.0-dev.2/README.md). Package version is still 0.2.0.

[Original dev.1 build downloads](https://github.com/danzig666/ps5-webos-remoteplay/tree/main/artifacts/v0.2.0-dev.1) include the native client IPK, the source archive with dependencies, the preserved M0 prototype, and SHA256SUMS. These are development builds with physical hardware testing deferred. The exact checksummed artifacts are stored in the repository because release-asset uploads were unavailable through the connected credentials.

## Build and install

Linux x86_64 prerequisites: Python 3.11+ with pip, curl, git, patch, make, gcc, node/npm, ar/readelf. Allow about 3 GB of tools/dependency space plus build caches. Exact tool, SDK, SDL archive and Git revisions are recorded in `tools.lock.json`, `upstream.lock.json` and `Cargo.lock`.

```bash
./scripts/install-tools.sh
export M0_TOOLS_DIR="$PWD/.tools"
export CARGO_HOME="$M0_TOOLS_DIR/cargo"
export RUSTUP_HOME="$M0_TOOLS_DIR/rustup"
export WEBOS_SDK_DIR="$M0_TOOLS_DIR/arm-webos-linux-gnueabi_sdk-buildroot"
export PATH="$CARGO_HOME/bin:$M0_TOOLS_DIR/python/cmake/data/bin:$M0_TOOLS_DIR/node/node_modules/.bin:$PATH"
export APPDATA="$M0_TOOLS_DIR/cli-data"  # optional writable CLI settings location
python3 scripts/fetch-upstream.py
./scripts/prepare-cargo.sh
./scripts/build-deps.sh
./scripts/build-client.sh
./scripts/package-client.sh
./scripts/verify-client.sh
./scripts/repro-client.sh
```

The client IPK is `dist/org.homebrew.ps5remote_0.2.0_arm.ipk`. The M0 feasibility package remains separate as `org.homebrew.ps5remote.m0`; it can coexist with the client. Its original package, source archive and historical evidence are preserved. Its earlier instructions are in [M0-build.md](docs/M0-build.md); current source builds use the workspace's Rust 1.99.1 pin.

Tool/source preparation requires network access; Cargo and native application builds then run offline against the lockfile. `verify-client.sh` uses loopback TCP/UDP sockets for the upstream protocol tests, never a PS5. [Hosted GitHub Actions passed](https://github.com/danzig666/ps5-webos-remoteplay/actions/runs/41397348283) for source commit `38032a8`, running the build, tests and checksum comparison. That hosted result covers the original release source. Current fixes and their local verification are documented in the bug-check report above.

Configure your TV in Developer Mode or WebOS Dev Manager, then, when hardware testing is resumed:

```bash
ares-install -d C2 dist/org.homebrew.ps5remote_0.2.0_arm.ipk
ares-launch -d C2 org.homebrew.ps5remote
./scripts/collect-client.sh C2 evidence/hardware/C2-client-run-01
```

No TV aliases/credentials are configured in this workspace. Do not interpret the build as an installation result. Standard Developer Mode is the intended privilege model. There are no root repairs, device chmods, console account changes or automatic exclusive input grabs.

## Use

Launch the app tile. Navigate with remote/gamepad arrows and OK/A, or a keyboard. Selecting a field opens a TV-local character keyboard; physical keyboard typing also works. Choose DONE or Back to close the editor. Account IDs, pairing PINs and login passcodes are masked and never logged.

1. Enter the PS5's private LAN IPv4 address. Hostnames and Internet addresses are rejected.
2. Enter the account ID as base64 encoding of its 8 bytes, and the eight-digit PIN from **PS5 Settings → System → Remote Play → Link Device**.
3. Select **PAIR / REPLACE PAIRING**, then **CONNECT**. Pairing is local, and successful keys are saved privately. A failed or cancelled pairing preserves prior credentials. Editing a different IP does not silently reuse another console's keys.
4. If the console requests its separate four-digit login passcode, enter it in the passcode field and select SUBMIT. Incorrect-passcode retries are controlled by the protocol's bounded flow; cancelling ends the attempt.

The console must already have Remote Play service and per-user permission enabled, with an offline-capable account on a supported jailbroken setup. Prepare that separately, using PS5Upload's documented procedure if needed. Back up the profile/saves before changing account IDs or offline activation settings; this client does not perform those operations. Disable conflicting encoder-blocking payloads. Firmware 13.60 is in the intended test matrix, not verified by this client build.

Start with H.264 SDR at 720p/1080p. Settings allow 30/60 fps, bitrate, stick dead zone, 10–80 ms audio jitter, mute, wake and bounded reconnect. Codec/resolution/fps/bitrate/jitter changes apply at the next connection; input and effect settings apply locally. Use the TV remote/system settings for volume and speaker/eARC routing. Audio is raw Opus into NDL; mute feeds a generated silent Opus packet of matching duration.

RED/F1 toggles the setup overlay while streaming. Opening the menu releases game input. Remote Back disconnects and shows the menu; Exit/quitting resets native effects and closes media. Gamepad buttons are forwarded to the PS5 while the menu is hidden. The controller's touchpad (one finger), click, gyro/accelerometer, sticks, triggers, and standard rumble use SDL capabilities when exposed. One active controller is selected; additional pads do not multiply player slots. Startup/hotplug/periodic scans cover already-paired Bluetooth pads and missing add events.

Reconnect is limited to five attempts with backoff and the upstream console-slot holdoff. It is used for transport/readiness failures, not generic authentication errors. Disconnect cancels pending retries. Backgrounding cancels the stream and releases input/effects; reconnect explicitly on return. Socket/wake startup calls retain upstream timeouts, so cancellation can leave a worker finishing for up to the wake/handshake timeout; the UI remains responsive and process exit waits at most two seconds.

## Optional M6 features

Attach the DualSense before CONNECT to negotiate its advanced stream channels. These are **off by default and unverified on hardware**:

- USB DualSense adaptive triggers/LEDs reuse Aurora's HID writer, with a small recorded intensity-byte patch. Off/strong/medium/weak honor the weaker user/console setting. The wrapper requires a USB GUID and exactly one wired DualSense to avoid the upstream fallback selecting another pad. Permission failures disable the path. Bluetooth adaptive triggers/HID output are not implemented or claimed.
- USB waveform haptics require explicitly choosing a controller audio device and opening exactly 48 kHz/s16le/four channels. Upsampled 3 kHz PCM goes to channels 3/4; channels 1/2 remain silent. No format fallback redirects this signal to TV speakers. If the output cannot open, PCM is converted to ordinary rumble. Device name/channel checks cannot establish the physical routing without a TV/controller test.
- Experimental HEVC HDR requests the upstream HDR stream, extracts received HEVC mastering-display/content-light SEI, and forwards PQ/BT.2020 metadata to NDL. It requires backend HDR capability and received mastering metadata. Missing metadata or decoder/HDR rejection closes that session and schedules HEVC SDR, then H.264 if needed. No fabricated mastering values, HDR success claim, 4K mode or software video fallback is provided.

## Architecture, evidence and source

Rust owns private configuration, lifecycle/UI state, pairing, stream workers, bounded queues, controller normalization, haptic conversion and HDR metadata parsing. C owns SDL/NDL lifecycle, rendering, input events and hardware output. Network callbacks never call SDL/NDL. Access units queue at most three frames; stale/dependent frames are dropped until an IDR and a keyframe is requested. Audio reorders wrapped sequence numbers in a bounded, paced jitter queue. Decoder recovery is limited to two restarts, with full parameter-set refeed; repeated failure stops the session.

The statistics distinguish received/submitted frames, queue drops and feed errors. Presented FPS, input-to-photon latency and physical effects remain unknown. Tests include upstream protocol fixtures/loopback peers, credential permission/symlink/cancellation checks, headless UI masking/input release, queue overflow/reordering, a two-hour **virtual-time** queue soak, HID report bytes in an in-memory sink, and package/ABI/reproducibility audits. A virtual soak does not measure real-time decoder performance.

Read [the hardware test matrix](docs/client-hardware-tests.md) and [capability CSV](docs/client-capability-matrix.csv), [implementation status](docs/implementation-status.md), and [source/license notices](THIRD_PARTY.md). Main logs are `/tmp/ps5remote.log` and one previous-launch log. Never attach `pairing.json` or unmasked account/PIN data to a report. Private data is under a private/ child of the SDL preference directory; `PS5REMOTE_DATA_DIR` is an optional explicit directory override for controlled diagnostics.

M0 remains GPLv3; the separate combined client and the new Rust app/protocol integration are AGPLv3. Pinned sources, recorded patches, dependency locks and license notices are provided for review/rebuilding. Source and the original development artifacts are published in the linked repository. No TV deployment, PS5 account modification or physical hardware test was performed.

The `ps5remote-webos-0.2.0-source.tar.gz` review archive includes the actual protocol/adapter sources, Cargo dependency sources and hash-manifested snapshots of the pinned native dependencies (including SDL3/webOS source). Source-snapshot verification works without Git metadata. The PS5Upload snapshot contains the exact original inputs used to verify the lean extraction; reproducing the optional full original-core portability experiment requires the full pinned upstream checkout. Toolchains/SDK archives are fetched separately using their recorded checksums.
