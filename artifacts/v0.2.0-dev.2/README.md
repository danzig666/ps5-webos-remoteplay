# Development build v0.2.0-dev.2

Includes all seven fixes from the [bug-check report](../../docs/bug-check-2026-11-02.md). Built from source commit `3c0594917fb22850c789855a95eac2186d54ebbf`; package version remains 0.2.0.

- [Download updated IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.2/artifacts/v0.2.0-dev.2/org.homebrew.ps5remote_0.2.0_arm.ipk)
- [SHA256SUMS](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.2/artifacts/v0.2.0-dev.2/SHA256SUMS)
- [Corresponding source and pinned build instructions](https://github.com/danzig666/ps5-webos-remoteplay/tree/3c0594917fb22850c789855a95eac2186d54ebbf)

Download the IPK and SHA256SUMS together, then run `sha256sum -c SHA256SUMS`.

147 Rust tests, native host sanitizer checks, ARM/package audits and two identical clean builds passed. Physical PS5/C2/C3 testing remains deferred. These are development artifacts; real hardware behavior is unverified.
