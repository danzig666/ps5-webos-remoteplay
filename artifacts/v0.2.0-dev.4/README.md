# C3 input diagnostics — v0.2.0-dev.4

C3 dev.3 evidence: configuration, SDL window/renderer and first frame submission succeeded. The user confirms the menu is visible but physical Up/Down do not move it. Input responsiveness is FAILED; streaming has not been tested. The previous log had no idle heartbeat and cannot distinguish missing remote events from a later blocked call.

This build corrects a confirmed mismatch: SDK SDL2 webOS Back/Exit/Red scancodes differ from the packaged SDL3 compatibility runtime. It reuses Aurora's pinned scancode definitions and handles standard navigation scancodes as well as key symbols. A host C regression test covers Up/Down/Left/Right/OK/Back/Exit/Red, symbol fallback and an unrelated character. This does not establish the cause of dead arrow keys on the TV.

Expanded bounded diagnostics include:

- SDL version, renderer backend, output/window dimensions, flags and keyboard focus.
- First 128 low-volume event types/window-state changes and 64 keyboard navigation results. No text-event content or character key values are printed.
- Separate-thread heartbeat every five seconds for the first minute: completed polls/events/navigation actions/frames, Rust loop phase, and current native call (controller scan, event poll, controller event, clear/fill/text/present or IME). A blocked UI thread can therefore still be diagnosed.
- First 16 render errors, with operation and SDL error. Existing startup, device-access and SS4S logs remain available.

**Retest:** install this IPK over the previous build, launch from the tile, press physical Down/Up/OK and Back, wait 30 seconds, then collect `/tmp/ps5remote.log` without relaunching first. If Home is needed to leave, collect afterward. Do not enter pairing secrets during this diagnostic run. Package version remains 0.2.0.

- [Download IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.4/artifacts/v0.2.0-dev.4/org.homebrew.ps5remote_0.2.0_arm.ipk)
- [SHA256SUMS](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.4/artifacts/v0.2.0-dev.4/SHA256SUMS)
- [Source](https://github.com/danzig666/ps5-webos-remoteplay/tree/v0.2.0-dev.4)

Local verification: 149 Rust tests, native navigation/HID/controller-selection/device-probe tests, ARM build, source-pin checks and full IPK audit pass. Hardware freeze remains unresolved pending this diagnostic log; this is not a claim of a working C3 client.
