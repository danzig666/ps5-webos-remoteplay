# M0 physical hardware verification — acceptance pending

Run every case on **both C2 and C3 independently**, recording the exact model suffix, webOS version, firmware, Developer Mode privilege context and package SHA-256. Updated firmware is a separate result; never infer C3 success from C2. Do not change console accounts or test PS5 connectivity during M0.

## Preparation and evidence

- [ ] Copy docs/run-template.md to a new evidence/hardware/<model>-<date>/run.md. Fill metadata before launch. Keep the original empty capability matrix as the untested baseline.
- [ ] Verify the downloaded IPK checksum against evidence/SHA256SUMS. Install using Developer Mode / WebOS Dev Manager. Record installation output and any dependency/permission errors. Do not enable root or chmod device nodes to make a test pass.
- [ ] Record a baseline with no gamepad: launcher tile opens an overlay, TV remote is not a controller slot, app exits after 120 seconds or Back. Collect /tmp/ps5remote-m0.log before relaunch; annotate missing logs rather than manufacturing one.

## Media and lifecycle

- [ ] Launch from the TV tile, without a PC service running. The bundled H.264 test pattern must move and repeat every 2 seconds. Capture a short video showing movement **and the overlay simultaneously**. Confirm the surface is transparent outside the diagnostic panel; a black surface is a failed compositor test even if feeds return zero.
- [ ] Save SDL runtime/renderer/alpha bits, SS4S init/open/feed return codes, NDL load/play callbacks and driver errors. An accepted feed or LOADCOMPLETED alone is insufficient proof of visible decode. Hardware-path evidence requires the selected ndl-webos5 module, successful NDL load and visible video together. No software decoder is packaged.
- [ ] Verify 120 seconds of playback without an expanding delay, freeze or crash; repeat launch/Back/launch three times. Record submitted/skipped/nonzero-feed counts from logs. Presented FPS and latency remain unknown unless independently measured. A 30-minute soak is outside this prototype's automatic 120-second run.
- [ ] Press Home/background and return. Record whether the app quits, resumes or stalls. Test termination during rumble, controller removal and relaunch. The motor must stop after at most the requested 500 ms pulse even if the process terminates. Record abnormal behavior.
- [ ] If there is no picture: retain the first failing log, firmware/model and a screenshot/video. Stop; do not retry initialization blindly, substitute a software decoder or escalate privileges. NDL ABI/foreground/exported-plane differences are hardware blockers for review.

## Controller matrix — repeat per connection and TV

Required rows: DualSense USB, DualSense Bluetooth, Xbox USB (if available), Xbox Bluetooth (if available). Stadia Bluetooth is optional. Missing hardware is **not tested**, not failed or passed.

- [ ] For USB: start already connected; then unplug/replug. For Bluetooth: pair using TV settings before app launch, then power off/on during the run. Remove the USB cable for the Bluetooth test. Note any webOS pairing limitation separately from client behavior.
- [ ] Confirm exactly one slot per physical pad. Match name/GUID/vendor/product and SDL path to bus logs: 0x0003=USB, 0x0005=Bluetooth. If no correlation is possible record transport as unknown, with physical connection stated separately.
- [ ] Exercise all face buttons, D-pad, shoulders, stick clicks, Options/Share, touchpad click and PS/Home. Record observed SDL button bits and OS interception. Do not claim touch coordinates, sensors or special effects from this M0 display.
- [ ] Sweep both sticks and triggers individually. Record idle/min/max, directions, analog trigger travel and unwanted coupled axes. Overlay axes use SDL order LX, LY, RX, RY, LT, RT. Compare the raw values with physical movements.
- [ ] Hold a button while disconnecting. The slot must clear; reconnect must begin released with no duplicate controller. Repeat five times per transport, including an already-paired startup scan.
- [ ] Press START/Options (or remote red / keyboard R). Record advertised capability, SDL rumble return code **and whether a human felt both motors**. API success with no physical output is not a rumble pass. Repeat for USB and Bluetooth independently.

## Device access and optional EVIOCGRAB

- [ ] Save each /dev/input/event* and /dev/hidraw* open result and errno. read_errno=0 means opened read-only; rw_errno=0 means opened read/write. ENOENT, EACCES, EPERM or no matching nodes are distinct observations. No HID output report is sent.
- [ ] Default run: verify no exclusive grab is attempted. Press G/green with M0_GRAB_DEVICE unset: expect `grab disabled`.
- [ ] Optional, only after identifying the **gamepad** event node by its bus/vendor/product: launch a diagnostic process with M0_GRAB_DEVICE=/dev/input/eventN and press G/green. Record the UID/euid and launch method; SSH permissions/foreground behavior may differ from the tile launch. Never select the TV remote/keyboard node. Expect acquire_rc and release_rc, or the exact permission error. The grab is immediate and momentary, so it establishes ioctl permission only; it does not prove interception avoidance during gameplay.
- [ ] If standard Developer Mode does not permit an environment override, mark the optional grab test **not tested: launch environment unavailable**. Do not alter package permissions or the TV to test it.

## Gate

M0 passes only after **both TVs** install/launch, visibly decode the fixture through NDL beneath the overlay, and provide USB/Bluetooth controller discovery plus device-access evidence under the intended privilege model. Record rumble and grab limitations truthfully; denial can be a measured limitation rather than a base-playback failure. Attach logs and physical observations, review blockers, then explicitly review M0 before considering M1. No M1 work is authorized by a build success.
