# M1–M6 implementation report

2 November 2026 — user explicitly authorized implementation through M6 with hardware testing deferred. This supersedes the earlier instruction to stop at M0 for hardware review. Historical M0 evidence remains valid only for its original build.

**Software implementation and ARM build evidence are available. No TV/PS5 acceptance gate is passed.** There has been no actual console pairing, streaming session, decoder presentation, physical controller test or native audio-device routing test.

| Milestone | Implemented software | Verification available | Deferred acceptance |
|---|---|---|---|
| M1 | Original PS5Upload core ARM portability build; lean pinned AGPL protocol extraction; Rust staticlib/C bootstrap; CI recipe; source/patch integrity checks | Original full core check and release library build; extracted protocol fixtures/loopback tests; native ELF link | TV launch, firmware ABI behavior, hosted CI run |
| M2 | TV-local IP/account/PIN entry; direct manual PIN registration; 0600 atomic key storage in 0700 directory; registration-key wake; console login passcode; state/cancel behavior | Account/LAN validation, storage permissions/symlinks, invalid-update preservation, cancelled pairing, upstream crypto and passcode tests | Offline PS5 account readiness, real pairing/wake/sign-in, firmware 13.60 |
| M3 | Direct upstream H.264/HEVC+Opus session; bounded video/audio queues; actual profile dimensions; transparent overlay; decoder retry/keyframe flow; start/stop | Cross-build, Annex-B/keyframe and wrapped audio ordering/pacing checks; no software video decoder | 1080p60 playback, NDL readiness, audio lip sync/eARC, real latency |
| M4 | Aurora SDL3/compat mappings and scans; USB/BT discovery; normalized axes/triggers/buttons; touch/motion; SDL rumble; GUID/input diagnostics; settings/reconnect | Controller mapping and release tests; queue/short-tap tests; upstream transport tests | DualSense/Xbox/Stadia matrix, sensor axes/signs, physical rumble, OS interception |
| M5 | Atomic credentials and redacted diagnostics; foreground/background cancellation; bounded retries/decoder recovery; input/effect reset; separate client IPK; reproducibility/CI scripts; source notices and test matrix | Host suite, headless UI secrecy/cancel/release checks, virtual two-hour queue soak, captured HID writer test, ABI/dependency/payload/checksum audits | Physical 30-minute/two-hour soaks, abnormal termination, TV sandbox, hosted CI and release review |
| M6 | USB HID triggers/LEDs with intensity; explicit four-channel USB audio haptics and fallback; experimental HDR metadata and SDR fallback; features disabled by default | HID byte layout/strength/reset captures, haptic channel isolation/interpolation, HDR SEI vectors, ARM link | hidraw permission, channel routing, actual effects, HDR colorimetry/plane/mode confirmation |

## Evidence

- `evidence/original-core-arm.txt` and `original-core-arm-build.txt`: original PS5Upload portability check and full release library build. The first release attempt required an explicit cross-linker for a transitive shared library; the recorded final build succeeds. The shipped client excludes archive/install/profile/desktop dependencies.
- `evidence/client-verification.txt`: 140 passing Rust tests (118 protocol, 19 application, 3 M0), plus native C checks. Rust tests including upstream TCP/UDP loopback responders, native HID report capture and device-probe negative tests, and package/ABI audits. The native UI test double is `#[cfg(test)]` only and is excluded from the release binary.
- `evidence/client-build.txt` and `client-package.txt`: native client link and package creation.
- `evidence/client-repro-before.txt`, `client-repro-after.txt` and `client-repro-check.txt`: same-path clean application/native-library rebuild comparisons. SDK/SDL prebuilt inputs are checksum-pinned. This does not prove reproducibility across different source paths/host environments; npm tool dependencies are separately locked and IPK metadata is canonicalized.
- `evidence/hardware/README.md`: no hardware runs have occurred. Matrix rows remain NOT_TESTED/DEFERRED.

- `evidence/client-source-tests.txt`, `client-source-deps.txt`, `client-source-arm-build.txt`, and `client-source-package.txt`: standalone exported-source test and ARM build/package, using an initially empty Cargo cache, vendored Rust dependency sources and hash-manifested native source snapshots. Locked dependency archives were pre-seeded for the offline dependency build. Different source paths produce different binary hashes; the same-path clean-build comparisons above are the reproducibility claim.

## Material limits

The native pipeline is executable code, not a mock, but hardware viability is still unknown. SDK NDL stubs link without proving firmware compatibility or compositor/exported-window behavior. SDL capability flags and API success are not physical effect evidence. Stereo Opus headers at 24/48 kHz are accepted; other audio layouts are rejected rather than misinterpreted. Volume/eARC routing uses TV settings.

Pairing requires the console's account to be offline-enabled and Remote Play-ready already. The client never alters users, account IDs, saves, Remote Play registry settings or firmware. It contains no Sony login/service client and only accepts private/link-local IPv4 console addresses. Automatic recovery does not overwrite credentials or switch users.

Initialization/wake/handshake operations use upstream socket timeouts; cancellation may finish asynchronously. Streaming cancellation sends released controller state before normal transport disconnect where possible. A broken LAN cannot guarantee delivery. Normal stop/foreground loss resets USB trigger resistance and SDL rumble; a process crash/SIGKILL cannot guarantee an HID reset, so abnormal-exit behavior needs the deferred controller test.

M6 output paths are explicitly experimental. USB association is conservative (one identifiable wired DualSense). Bluetooth advanced HID effects remain outside the implementation. Four-channel USB output still needs physical endpoint/routing verification. HDR uses received mastering metadata plus the upstream HDR contract (PQ/BT.2020); absent metadata/rejected setup schedules SDR rather than displaying guessed HDR.

No native 4K stream, PC relay, software video transcoder, account-modification wizard, provisioning export/import, microphone uplink or console on-screen-keyboard proxy is added. Those are not required by the M1–M6 plan's native local-pairing/streaming scope; local app text entry and console login passcodes are implemented.

## Next review

Review the code and packaged build now. Resume the deferred C2/C3 and offline PS5 matrix when hardware becomes available, beginning with H.264 SDR, then controllers/audio/reconnect, then M6 features individually. A failed hardware gate should be diagnosed from its first concrete log/capture; do not infer success or perform privileged repairs. M0–M6 cannot be called hardware-accepted or release-ready until those results exist.
