# Deferred native-client acceptance matrix

**Every item below is NOT TESTED.** Run separately on C2 and C3 and each updated firmware. Record model suffix, webOS/firmware, Developer Mode privilege context, package SHA-256, console firmware (including 13.60 where available), controller/transport and evidence file/time. Do not collect secret configuration.

| Gate | C2 | C3 | Required evidence |
|---|---|---|---|
| Native install/tile launch/transparent video plane | NOT TESTED | NOT TESTED | Install output, SDL/NDL log, moving video with overlay |
| Local PIN pairing and private storage | NOT TESTED | NOT TESTED | Redacted outcome and file mode only; never keys/account/PIN |
| Direct offline wake/connect/stream with Internet blocked | NOT TESTED | NOT TESTED | LAN-only firewall observation, no PC service, console status |
| Four-digit login passcode/cancel/wrong-code flow | NOT TESTED | NOT TESTED | Redacted UI outcome, no passcode recording |
| H.264 720p60/1080p60, 30-minute and two-hour soak | NOT TESTED | NOT TESTED | Video/audio observations, loss/feed/queue counters, measured presented FPS if available |
| Opus, mute, TV volume, speakers/eARC | NOT TESTED | NOT TESTED | Lip-sync/output-route observations and native audio errors |
| DualSense USB and Bluetooth input/rumble/reconnect | NOT TESTED | NOT TESTED | Buttons/axes/triggers/touch/motion, five reconnects, physically felt motors |
| Xbox USB/Bluetooth; optional Stadia Bluetooth | NOT TESTED | NOT TESTED | Per-device identity/transport and per-feature results |
| No stuck input after unplug/menu/background/exit | NOT TESTED | NOT TESTED | Hold/disconnect/reconnect sequence and console observations |
| Network interruption/reconnect/slot holdoff/cancel | NOT TESTED | NOT TESTED | Timed disconnect/retry logs, stable credentials |
| HEVC SDR and unsupported-decoder fallback | NOT TESTED | NOT TESTED | Selected mode, NDL results, visible picture |
| USB HID permission, triggers, LEDs, all strengths/off | NOT TESTED | NOT TESTED | Accessible node identity, write result AND physical effects |
| USB four-channel haptics and ordinary rumble fallback | NOT TESTED | NOT TESTED | Explicit endpoint name/format, silent speaker channels, felt waveform, fallback behavior |
| Experimental HDR and SDR fallback | NOT TESTED | NOT TESTED | Received mastering metadata, NDL result, TV HDR mode/color comparison, absent-metadata/rejection behavior |
| Normal exit, background, crash/relaunch during effects | NOT TESTED | NOT TESTED | Trigger release/motor timeout; document crash limits rather than claiming cleanup |
| Memory, input-to-photon latency and sustained presented FPS | NOT TESTED | NOT TESTED | TV RSS/queue trend and external high-speed-camera measurement |

Use `scripts/collect-client.sh DEVICE OUTPUT_DIR` for the app log and device metadata. Attach a copy of docs/run-template.md updated for client 0.2.0. Log counters are received/submitted/feed statistics; they do not establish display FPS or latency. Keep physical pass/fail separate from API return values.

No root/chmod changes or automatic EVIOCGRAB are required or attempted. The original M0 package retains its separate explicitly selected, immediately released EVIOCGRAB permission probe. An SSH-launched diagnostic process may have different permissions/foreground state from a Developer Mode tile launch; label results accordingly.
