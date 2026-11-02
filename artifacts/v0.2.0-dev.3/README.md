# C3 startup correction — v0.2.0-dev.3

The user-provided C3 log shows `Client error: Unsafe data directory`, before SDL initialization. The pinned SDL webOS GetPrefPath implementation returns the installation directory; the application incorrectly applied private-storage ownership checks to that installer-owned directory.

This build creates an app-owned `private/` child for settings and credentials. It preserves directory/file ownership and permission checks and does not chmod the installation directory. Existing explicit PS5REMOTE_DATA_DIR overrides retain their meaning.

Two additional startup mismatches with pinned Aurora were corrected: remove SDL_WINDOW_FULLSCREEN_DESKTOP (unsupported fullscreen transition for its webOS surface), and stop automatic text input until an editor is visible. The log did not reach these paths, so they are not presented as the cause of this particular failure. SS4S/NDL initialization order was investigated but left unchanged: the packaged backend defers NDL initialization until playback.

Startup stage/error messages now identify configuration, SDL initialization, window/renderer creation, controller probing and the first frame submission. They do not log pairing credentials or typed input.

- [Download IPK](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.3/artifacts/v0.2.0-dev.3/org.homebrew.ps5remote_0.2.0_arm.ipk)
- [SHA256SUMS](https://raw.githubusercontent.com/danzig666/ps5-webos-remoteplay/v0.2.0-dev.3/artifacts/v0.2.0-dev.3/SHA256SUMS)
- [Source](https://github.com/danzig666/ps5-webos-remoteplay/tree/v0.2.0-dev.3)

Package version remains 0.2.0. Install this IPK over the old development build using Dev Manager/ares-install, then launch from the TV tile. If it still fails, collect `/tmp/ps5remote.log` immediately after that launch. Do not uninstall solely to install this build, since uninstalling may remove app data.

ARM build and IPK audit passed. 149 Rust tests passed across the full verification and the final app-only regression run (27 app, 119 protocol, 3 M0). Regression coverage includes private subdirectory creation preserving parent permissions and text input restricted to a visible editor. These host checks do not verify a successful C3 launch. The supplied dev.2 failure is recorded; dev.3 hardware retest is pending.
