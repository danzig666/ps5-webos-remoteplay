# C3 keyboard event ABI correction — v0.2.0-dev.5

The dev.4 C3 log establishes that the UI is not deadlocked: after 45 seconds, 1,941 frames and 15,931 event polls completed. The window has keyboard focus and key-down events arrive, but all navigation results are zero.

Root cause in the build: SDK SDL2 headers add a webOS `inputSource` field to `SDL_KeyboardEvent`. The shipped sdl2-compat library emits the standard SDL2 structure. The client consequently reads keysym at byte 20 instead of byte 16, repeat at byte 17 instead of byte 13, and expects a 36-byte keyboard event instead of 32 bytes. Arrow keys therefore become unrecognized values, and scancode bytes can appear as spurious repeat flags. The previous navigation test constructed events with the same incorrect headers and missed this mismatch.

The native client and current M0 source now use the headers installed alongside the pinned compatibility runtime by `scripts/build-deps.sh`, including its generated Unix configuration header. SDK headers are no longer used for event handling. M0 remote constants also use pinned Aurora definitions. Existing versioned M0 download artifacts are unchanged.

Regression protection: compile-time checks for keyboard size/key/repeat offsets; a test decoding an independently specified sdl2-compat byte-layout fixture; and a deliberate wrong-SDK-header compilation that fails all three ABI assertions. Native host tests now use the matching headers too.

- [Download corrected IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.5/artifacts/v0.2.0-dev.5/org.homebrew.ps5remote_0.2.0_arm.ipk)
- [SHA256SUMS](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.5/artifacts/v0.2.0-dev.5/SHA256SUMS)
- [Corresponding source](https://github.com/danzig666/ps5-webos-remoteplay/tree/v0.2.0-dev.5)

Install over the previous development build, launch from the tile, then try physical Up/Down and OK. If navigation still fails, wait 30 seconds and collect `/tmp/ps5remote.log` before relaunching. Expanded bounded diagnostics from dev.4 remain active. Package version remains 0.2.0.

Validation: 149 Rust tests, native navigation/HID/controller-selection/device-probe tests, host address/undefined-behavior sanitizers (leak detection disabled under tracing), ARM build and IPK audit passed. Actual C3 navigation with this build awaits retest. No PS5 streaming or controller-output success is claimed. Evidence is in `evidence/C3-event-abi` and `evidence/hardware/C3-startup/dev4-ps5remote.log`.
