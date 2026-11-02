> Historical M0 report. The user subsequently authorized M1–M6 implementation with hardware verification deferred; see [current status](implementation-status.md). Physical claims below remain unchanged.

# M0 implementation report — 2 November 2026

**Prototype implemented and cross-build verified. Hardware feasibility acceptance is pending. No C2 or C3 was connected, installed, launched or physically tested. Do not proceed to M1.**

## Confirmed in this environment

- Rust 1.85.1 staticlib and C11 bootstrap/platform adapters cross-compile for ARMv7 Linux soft-float using the checksum-pinned buildroot-nc4 2026.08-webos SDK (GCC 16.2.0).
- The pinned, unmodified Aurora SS4S core and NDL webOS5 implementation build and link against the SDK NDL stub with no undefined module references. The Rust harness opens only 1080p60 H.264 video and feeds complete AUD-delimited Annex-B access units with keyframe flags. There is no software video decoder.
- Aurora's actual SDL3/webOS stack, SDL2 compatibility layer, webOS scancode/private-SONAME patches and NDL exported-window shim are packaged. The SDK's dummy-only SDL library was detected during audit and removed before handoff. No dummy media backend is present.
- The package includes startup/reconnect scans, instance-ID deduplication, Aurora gamepad filtering/mappings, button/axis diagnostics, timed rumble requests, event/hidraw read/read-write probes, and explicitly opt-in momentary EVIOCGRAB with release/close.
- Three Rust parser/fixture tests pass. The checked-in fixture software-decodes as 120 H.264 frames at 1920×1080/60 with no B frames (this verifies the asset, not the TV decoder). C negative-path checks verify grab is disabled without an explicit valid event path and reports missing devices.
- ELF32 ARM, soft-float ABI, NDL module selection, packaged dependency closure, asset contents and canonical IPK payload checks pass. Two clean application/native-module/SDL-compatibility builds and package runs produce identical checksums; see evidence/repro-before.txt and evidence/repro-after.txt. This is same-path build reproducibility, not proof across different hosts/toolchain locations. Canonical ar/tar/gzip metadata removes CLI timestamps; the checked-in fixture avoids encoder-version variability.

Evidence: evidence/build.txt, dependencies.txt, verification.txt, package.txt, rebuild.txt, repackage.txt repro-check.txt and SHA256SUMS. Hardware evidence directory explicitly records that no physical run has occurred.

## Remaining blockers and limitations

1. **C2/C3 hardware access is required.** No claims about NDL permission, hardware-decoded presentation, compositor blending, firmware ABI compatibility, controller access, rumble output or EVIOCGRAB success can be made from cross-compilation. Every hardware row is NOT_TESTED in docs/capability-matrix.csv.
2. NDL headers/stubs and bundled SDL implementations compile, but actual TV library/foreground/exported-plane behavior must be checked under standard Developer Mode on each firmware. The package intentionally does not bundle TV-owned NDL/EGL/libc or change privileges. Unknown/denied device access is logged as such.
3. The harness reports submitted/skipped/feed-result counts, not measured decoded/presented FPS or end-to-end latency. It uses one thread for SDL/NDL and synchronous local feeding; lengthy native calls can delay diagnostics. This intentionally small feasibility experiment is not the later streaming queue/thread architecture.
4. SDL's advertised rumble capability and a zero return code do not prove vibration. A human must verify USB and Bluetooth independently. Momentary grab proves only ioctl permission; it does not prove prevention of OS shortcuts. Touch coordinates, motion, advanced feedback and console functions are excluded.
5. The exact SDL3 release binary is archive-pinned and compatibility sources are Git-pinned. Tool installation needs network; normal Cargo/app builds are offline after dependency preparation. Retain corresponding upstream sources/notices if distributing beyond the local hardware trial. npm transitive tools are locked separately.

## Next action and milestone

Install the supplied IPK on each physical TV and follow docs/hardware-checklist.md. Collect actual logs with scripts/collect-tv.sh and complete a run record, firmware-specific matrix and moving-video/physical-rumble observations. Stop on the first meaningful decoder/compositor/permission failure and bring its evidence to M0 review.

Only after M0 is reviewed and the base hardware gate passes should M1 be considered: lean ARMv7 PS5Upload core portability, protocol fixtures and lifecycle integration. No protocol, offline-account changes or pairing code is implemented here.
