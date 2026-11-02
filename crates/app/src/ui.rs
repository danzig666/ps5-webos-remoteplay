// SPDX-License-Identifier: AGPL-3.0-only
use crate::{
    config::{validate_account, validate_host, Pairing, Preferences, Store},
    ffi::{self, Event, RawInput},
    haptics::Upsampler,
    session::{self, lock, Outcome, Shared},
};
use psremote::rp_stream::{ControllerState, Passcode};
use std::{
    ffi::CString,
    sync::{atomic::Ordering, mpsc, Arc},
    thread::JoinHandle,
    time::{Duration, Instant},
};
const ITEMS: usize = 26;
const KEYS: &str = "0123456789.ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/=";
struct Editor {
    field: usize,
    cursor: usize,
}
pub struct App {
    pub store: Store,
    pub prefs: Preferences,
    pub paired: Option<Pairing>,
    pub host: String,
    account: String,
    pin: String,
    login: String,
    active_codec: u8,
    active_fps: u32,
    rumble_is_haptics: bool,
    last_rumble_refresh: Instant,
    outputs_dirty: bool,
    selected: usize,
    editor: Option<Editor>,
    menu: bool,
    message: String,
    muted: bool,
    raw: RawInput,
    shared: Option<Arc<Shared>>,
    worker: Option<JoinHandle<()>>,
    pair_rx: Option<mpsc::Receiver<Result<Pairing, &'static str>>>,
    pair_cancelled: bool,
    decoder: Option<(u32, u32)>,
    decoder_failures: u32,
    last_video: Instant,
    last_received: Instant,
    retry_at: Option<Instant>,
    retries: u32,
    fallback: Option<u8>,
    audio_devices: Vec<String>,
    haptics_open: bool,
    haptics_attempted: bool,
    upsampler: Upsampler,
    last_pcm: Instant,
    last_rumble: (u8, u8),
    raw_rumble: (u8, u8),
    rumble_time: Instant,
    last_triggers: Option<[u8; 22]>,
    last_led: Option<[u8; 3]>,
    last_intensity: (u8, u8),
}
impl App {
    pub fn new(store: Store, prefs: Preferences, paired: Option<Pairing>) -> Self {
        let now = Instant::now();
        Self {
            host: prefs.host.clone(),
            active_codec: prefs.codec,
            active_fps: prefs.fps,
            rumble_is_haptics: false,
            last_rumble_refresh: now,
            outputs_dirty: true,
            prefs,
            paired,
            store,
            account: String::new(),
            pin: String::new(),
            login: String::new(),
            selected: 0,
            editor: None,
            menu: true,
            message: "Configure LAN IP and pair locally. No Sony sign-in.".into(),
            muted: false,
            raw: RawInput::default(),
            shared: None,
            worker: None,
            pair_rx: None,
            pair_cancelled: false,
            decoder: None,
            decoder_failures: 0,
            last_video: now,
            last_received: now,
            retry_at: None,
            retries: 0,
            fallback: None,
            audio_devices: Vec::new(),
            haptics_open: false,
            haptics_attempted: false,
            upsampler: Upsampler::default(),
            last_pcm: now,
            last_rumble: (0, 0),
            raw_rumble: (0, 0),
            rumble_time: now,
            last_triggers: None,
            last_led: None,
            last_intensity: (1, 1),
        }
    }
    fn notice(&mut self, s: &str) {
        self.message = s.into();
    }
    fn clear_outputs(&mut self) {
        unsafe {
            ffi::rp_effects_reset();
            ffi::rp_media_close();
        }
        self.decoder = None;
        self.haptics_open = false;
        self.haptics_attempted = false;
        self.last_rumble = (0, 0);
        self.raw_rumble = (0, 0);
        self.last_led = None;
        self.last_triggers = None;
        self.upsampler = Upsampler::default();
    }
    fn disconnect(&mut self) {
        if let Some(s) = &self.shared {
            s.cancel();
        }
        self.retry_at = None;
        self.fallback = None;
        self.pair_cancelled = true;
        self.clear_outputs();
        self.menu = true;
        self.notice("Disconnecting; credentials preserved");
    }
    fn connect(&mut self, automatic: bool) {
        if self.worker.is_some() || self.pair_rx.is_some() {
            self.notice("Previous operation is still finishing");
            return;
        }
        if validate_host(&self.host).is_err() {
            self.notice("Enter a private LAN IPv4 address");
            return;
        }
        let Some(pair) = self.paired.clone() else {
            self.notice("Pair locally before connecting");
            return;
        };
        if pair.host != self.host {
            self.notice("Saved pairing belongs to another IP; pair this console first");
            return;
        }
        self.prefs.host = self.host.clone();
        self.active_codec = self.prefs.codec;
        self.active_fps = self.prefs.fps;
        let shared = Shared::new(&self.prefs);
        self.worker = Some(session::start(
            pair,
            self.prefs.clone(),
            self.raw.connected == 2,
            shared.clone(),
        ));
        self.shared = Some(shared);
        self.menu = false;
        self.last_video = Instant::now();
        self.last_received = Instant::now();
        self.decoder_failures = 0;
        self.haptics_attempted = false;
        if !automatic {
            self.retries = 0;
        }
        self.notice("Connecting directly over LAN");
    }
    fn pair(&mut self) {
        if self.worker.is_some() || self.pair_rx.is_some() {
            self.notice("Disconnect before pairing");
            return;
        }
        if let Err(e) = validate_host(&self.host).and_then(|_| validate_account(&self.account)) {
            self.notice(e);
            return;
        }
        if psremote::rp_regist::parse_manual_pin(&self.pin).is_err() {
            self.notice("Enter the eight-digit Link Device PIN");
            return;
        }
        let host = self.host.clone();
        let account = std::mem::take(&mut self.account);
        let pin = std::mem::take(&mut self.pin);
        let (tx, rx) = mpsc::sync_channel(1);
        self.pair_rx = Some(rx);
        self.pair_cancelled = false;
        self.notice("Pairing locally; existing credentials remain until success");
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                psremote::rp_regist::pair_with_pin(&host, &account, &pin)
            });
            let result = match result {
                Ok(Ok(r)) => Ok(Pairing {
                    version: 1,
                    host,
                    regist_key: r.regist_key,
                    rp_key: r.rp_key,
                }),
                _ => Err("Pairing failed. Check user ID, Link Device PIN, and console readiness."),
            };
            let _ = tx.send(result);
        });
    }
    fn field(&mut self, i: usize) -> &mut String {
        match i {
            0 => &mut self.host,
            1 => &mut self.account,
            2 => &mut self.pin,
            _ => &mut self.login,
        }
    }
    fn append(&mut self, i: usize, text: &str) {
        let max = match i {
            0 => 15,
            1 => 24,
            2 => 8,
            _ => 4,
        };
        let target = self.field(i);
        for c in text.chars() {
            let valid = match i {
                0 => c.is_ascii_digit() || c == '.',
                1 => c.is_ascii_alphanumeric() || "+/=".contains(c),
                _ => c.is_ascii_digit(),
            };
            if valid && target.len() < max {
                target.push(c);
            }
        }
    }
    fn adjust(&mut self, direction: i32) {
        let forward = direction >= 0;
        match self.selected {
            6 => self.prefs.codec = (self.prefs.codec + if forward { 1 } else { 2 }) % 3,
            7 => self.prefs.height = if self.prefs.height == 1080 { 720 } else { 1080 },
            8 => self.prefs.fps = if self.prefs.fps == 60 { 30 } else { 60 },
            9 => {
                self.prefs.bitrate = (self.prefs.bitrate as i32
                    + if forward { 1000 } else { -1000 })
                .clamp(2000, 50000) as u32
            }
            10 => {
                self.prefs.deadzone = (self.prefs.deadzone as i32
                    + if forward { 500 } else { -500 })
                .clamp(0, 16000) as i16
            }
            11 => {
                self.prefs.jitter_ms = (self.prefs.jitter_ms as i32
                    + if forward { 10 } else { -10 })
                .clamp(10, 80) as u64
            }
            12 => {
                self.prefs.reconnect = !self.prefs.reconnect;
                if !self.prefs.reconnect {
                    self.retry_at = None;
                    self.fallback = None;
                }
            }
            13 => self.prefs.wake = !self.prefs.wake,
            14 => {
                self.prefs.usb_effects = !self.prefs.usb_effects;
                unsafe {
                    ffi::rp_effects_reset();
                }
                self.haptics_open = false;
                self.haptics_attempted = false;
            }
            15 => {
                self.prefs.usb_audio = !self.prefs.usb_audio;
                unsafe {
                    ffi::rp_haptics_close();
                }
                self.haptics_open = false;
                self.haptics_attempted = false;
            }
            16 => {
                self.audio_devices.clear();
                for i in 0..unsafe { ffi::rp_audio_device_count() } {
                    let mut b = [0 as std::ffi::c_char; 256];
                    unsafe {
                        ffi::rp_audio_device_name(i, b.as_mut_ptr(), b.len());
                    }
                    let bytes: Vec<u8> =
                        b.iter().map(|x| *x as u8).take_while(|x| *x != 0).collect();
                    let name = String::from_utf8_lossy(&bytes).into_owned();
                    if name.contains("DualSense") || name.contains("Wireless Controller") {
                        self.audio_devices.push(name);
                    }
                }
                self.audio_devices.push(String::new());
                let old = self
                    .audio_devices
                    .iter()
                    .position(|n| n == &self.prefs.haptics_device)
                    .unwrap_or(self.audio_devices.len() - 1);
                self.prefs.haptics_device =
                    self.audio_devices[(old + 1) % self.audio_devices.len()].clone();
                unsafe {
                    ffi::rp_haptics_close();
                }
                self.haptics_open = false;
                self.haptics_attempted = false;
            }
            17 => self.prefs.haptics_strength = (self.prefs.haptics_strength + 1) % 4,
            18 => self.prefs.trigger_strength = (self.prefs.trigger_strength + 1) % 4,
            19 => self.muted = !self.muted,
            _ => return,
        }
        self.notice(
            "Changed locally; Save settings to persist. Codec/resolution apply on next connect.",
        );
    }
    pub fn event(&mut self, e: &Event) -> bool {
        if e.input_changed != 0 {
            self.raw = e.input;
            if self.raw.connected == 0 {
                unsafe {
                    ffi::rp_effects_reset();
                }
                self.haptics_open = false;
                self.haptics_attempted = false;
                self.last_rumble = (0, 0);
            }
            if let Some(s) = &self.shared {
                lock(&s.inputs).push(if self.menu {
                    ControllerState::default()
                } else {
                    self.raw.normalized(&self.prefs)
                });
            }
        }
        if e.kind == 9 {
            return false;
        }
        if e.kind == 10 {
            self.disconnect();
            self.notice("Paused in background. Connect to resume.");
            return true;
        }
        if e.kind == 11 {
            self.menu = !self.menu;
            if let Some(s) = &self.shared {
                lock(&s.inputs).push(ControllerState::default());
            }
            return true;
        }
        if !self.menu {
            if e.kind == 6 && e.value == 1 {
                self.disconnect();
            }
            return true;
        }
        if let Some(mut editor) = self.editor.take() {
            let n = KEYS.len() + 2;
            match e.kind {
                1 => editor.cursor = editor.cursor.saturating_sub(10),
                2 => editor.cursor = (editor.cursor + 10).min(n - 1),
                3 => editor.cursor = (editor.cursor + n - 1) % n,
                4 => editor.cursor = (editor.cursor + 1) % n,
                5 => {
                    if editor.cursor == KEYS.len() {
                        self.field(editor.field).pop();
                    } else if editor.cursor == KEYS.len() + 1 {
                        return true;
                    } else {
                        let c = KEYS.as_bytes()[editor.cursor] as char;
                        self.append(editor.field, &c.to_string());
                    }
                }
                6 => return true,
                7 => {
                    let bytes: Vec<u8> = e.text.iter().copied().take_while(|b| *b != 0).collect();
                    self.append(editor.field, &String::from_utf8_lossy(&bytes));
                }
                8 => {
                    self.field(editor.field).pop();
                }
                _ => {}
            }
            self.editor = Some(editor);
            return true;
        }
        match e.kind {
            1 => self.selected = (self.selected + ITEMS - 1) % ITEMS,
            2 => self.selected = (self.selected + 1) % ITEMS,
            3 => self.adjust(-1),
            4 => self.adjust(1),
            6 => {
                if self.worker.is_some() {
                    self.menu = false;
                } else {
                    return false;
                }
            }
            5 => match self.selected {
                0..=2 | 20 => {
                    if self.pair_rx.is_some() {
                        self.notice("Finish or cancel the pending pairing first");
                        return true;
                    }
                    self.editor = Some(Editor {
                        field: self.selected,
                        cursor: 0,
                    });
                }
                3 => self.pair(),
                4 => self.connect(false),
                5 => self.disconnect(),
                6..=19 => self.adjust(1),
                21 => {
                    if psremote::rp_stream::valid_passcode(&self.login) {
                        if let Some(s) = &self.shared {
                            *lock(&s.passcode) =
                                Some(Passcode::Digits(std::mem::take(&mut self.login)));
                            s.passcode_requested.store(false, Ordering::Release);
                            self.menu = false;
                        }
                    } else {
                        self.notice("Console passcode must be four digits");
                    }
                }
                22 => {
                    self.prefs.host = self.host.clone();
                    match self.store.save_preferences(&self.prefs) {
                        Ok(()) => self.notice("Settings saved privately"),
                        Err(e) => self.notice(e),
                    }
                }
                23 => {
                    unsafe {
                        ffi::rp_feedback(
                            80,
                            160,
                            std::ptr::null(),
                            std::ptr::null(),
                            self.prefs.usb_effects as i32,
                            self.prefs.trigger_strength as i32,
                        );
                    }
                    self.notice("Rumble requested; verify physically. No success assumed.");
                }
                24 => self.menu = false,
                25 => return false,
                _ => {}
            },
            _ => {}
        }
        true
    }
    pub fn update(&mut self) {
        if let Some(rx) = &self.pair_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.pair_rx = None;
                    if !self.pair_cancelled {
                        match result {
                            Ok(pair) => match self.store.save_pairing(&pair) {
                                Ok(()) => {
                                    self.prefs.host = pair.host.clone();
                                    self.host = pair.host.clone();
                                    self.paired = Some(pair);
                                    let _ = self.store.save_preferences(&self.prefs);
                                    self.notice(
                                        "Paired locally. Keys saved privately. Select Connect.",
                                    );
                                }
                                Err(e) => self.notice(e),
                            },
                            Err(e) => self.notice(e),
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pair_rx = None;
                    self.notice("Pairing worker stopped; credentials unchanged");
                }
                Err(_) => {}
            }
        }
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            let _ = self.worker.take().unwrap().join();
            let outcome = self
                .shared
                .as_ref()
                .map(|s| *lock(&s.outcome))
                .unwrap_or(Outcome::Stopped);
            self.clear_outputs();
            self.menu = true;
            if let Some(codec) = self.fallback.take() {
                self.prefs.codec = codec;
                self.retry_at = Some(Instant::now() + Duration::from_secs(36));
                self.notice("Decoder fallback: reconnecting with SDR after console release");
            } else if outcome == Outcome::Transient && self.prefs.reconnect && self.retries < 5 {
                self.retries += 1;
                self.retry_at = Some(
                    Instant::now() + Duration::from_secs((35 + 2u64.pow(self.retries)).min(95)),
                );
                self.notice("LAN interrupted. Reconnect scheduled; Back/Disconnect cancels.");
            } else {
                self.retry_at = None;
            }
        }
        if self.retry_at.is_some_and(|t| Instant::now() >= t) {
            self.retry_at = None;
            self.connect(true);
        }
        let Some(shared) = self.shared.clone() else {
            return;
        };
        if shared.passcode_requested.swap(false, Ordering::AcqRel) {
            self.menu = true;
            self.selected = 20;
        }
        if self.worker.is_none() || shared.stop.load(Ordering::Acquire) {
            return;
        }
        self.pump_media(&shared);
        self.pump_feedback(&shared);
    }
    fn fallback(&mut self, s: &Shared, codec: u8, message: &str) {
        s.cancel();
        self.fallback = Some(codec);
        self.clear_outputs();
        self.menu = true;
        self.notice(message);
    }
    fn pump_media(&mut self, s: &Shared) {
        let now = Instant::now();
        let header = lock(&s.audio).header;
        let metadata = *lock(&s.metadata);
        if self.active_codec == 2 && metadata.is_none() {
            if s.received.load(Ordering::Relaxed) > 0
                && self.last_received.elapsed() > Duration::from_secs(3)
            {
                self.fallback(s, 1, "HDR metadata unavailable; falling back to HEVC SDR");
            }
            return;
        }
        let frame = {
            let mut q = lock(&s.video);
            let f = q.pop(now);
            if q.waiting {
                s.keyframe.store(true, Ordering::Relaxed);
            }
            f
        };
        if let Some(frame) = frame {
            self.last_received = now;
            let Some(h) = header else {
                if now.duration_since(self.last_video) > Duration::from_secs(3) {
                    s.cancel();
                    self.notice("Unsupported or missing Opus audio header");
                    self.menu = true;
                }
                return;
            };
            if self.decoder != Some((frame.width, frame.height)) {
                let rc = unsafe {
                    ffi::rp_media_open(
                        frame.width as i32,
                        frame.height as i32,
                        self.active_fps as i32,
                        self.active_codec as i32,
                        h.rate as i32,
                        h.channels as i32,
                        h.frame_size as i32,
                        metadata
                            .as_ref()
                            .map_or(std::ptr::null(), |m| m as *const _),
                    )
                };
                if rc != 0 {
                    if self.active_codec != 0 {
                        self.fallback(
                            s,
                            if self.active_codec == 2 { 1 } else { 0 },
                            "Requested decoder mode unavailable; scheduling SDR fallback",
                        );
                    } else {
                        s.cancel();
                        self.notice("NDL decoder open failed; inspect TV logs");
                        self.menu = true;
                    }
                    return;
                }
                self.decoder = Some((frame.width, frame.height));
                self.last_video = now;
            }
            let rc =
                unsafe { ffi::rp_video(frame.data.as_ptr(), frame.data.len(), frame.key as i32) };
            if rc == 0 {
                s.submitted.fetch_add(1, Ordering::Relaxed);
                self.last_video = now;
            } else {
                s.feed_errors.fetch_add(1, Ordering::Relaxed);
                s.keyframe.store(true, Ordering::Relaxed);
                lock(&s.video).reset();
            }
        }
        if self.decoder.is_some() && self.last_video.elapsed() > Duration::from_secs(2) {
            self.decoder_failures += 1;
            unsafe {
                ffi::rp_media_close();
            }
            self.decoder = None;
            lock(&s.video).reset();
            s.keyframe.store(true, Ordering::Relaxed);
            self.last_video = now;
            if self.decoder_failures > 2 {
                s.cancel();
                self.menu = true;
                self.notice("Decoder stalled repeatedly; stopped for diagnostics");
            }
        }
        if self.decoder.is_some() {
            for _ in 0..4 {
                let audio = lock(&s.audio).pop(now);
                let Some(packet) = audio else {
                    break;
                };
                if unsafe { ffi::rp_audio(packet.as_ptr(), packet.len(), self.muted as i32) } != 0 {
                    s.audio_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
    fn pump_feedback(&mut self, s: &Shared) {
        let feedback = {
            let mut f = lock(&s.feedback);
            let out = f.clone();
            f.rumble = None;
            f.led = None;
            f.triggers = None;
            f.pcm.clear();
            out
        };
        if let Some(rgb) = feedback.led {
            self.last_led = Some(rgb);
        }
        if let Some((l, r)) = feedback.triggers {
            let mut b = [0u8; 22];
            b[0] = l.0;
            b[1..11].copy_from_slice(&l.1);
            b[11] = r.0;
            b[12..22].copy_from_slice(&r.1);
            self.last_triggers = Some(b);
        }
        let h_strength = weaker(self.prefs.haptics_strength, feedback.intensity.0);
        let t_strength = weaker(self.prefs.trigger_strength, feedback.intensity.1);
        let refresh = self.outputs_dirty;
        self.outputs_dirty = false;
        let mut changed = refresh
            || feedback.rumble.is_some()
            || feedback.led.is_some()
            || feedback.triggers.is_some()
            || self.last_intensity != (h_strength, t_strength);
        let intensity_changed = self.last_intensity != (h_strength, t_strength);
        self.last_intensity = (h_strength, t_strength);
        if let Some((l, r)) = feedback.rumble {
            self.rumble_is_haptics = false;
            self.raw_rumble = (l, r);
            self.last_rumble = scale_rumble((l, r), h_strength);
            self.rumble_time = Instant::now();
        }
        if !self.rumble_is_haptics && intensity_changed {
            self.last_rumble = scale_rumble(self.raw_rumble, h_strength);
        }
        if h_strength == 0 && self.haptics_open {
            unsafe {
                ffi::rp_haptics_close();
            }
            self.haptics_open = false;
            self.haptics_attempted = false;
        }
        if h_strength != 0
            && self.prefs.usb_audio
            && !self.haptics_attempted
            && self.raw.connected != 0
        {
            self.haptics_attempted = true;
            let name = CString::new(self.prefs.haptics_device.as_str()).unwrap_or_default();
            self.haptics_open = unsafe { ffi::rp_haptics_open(name.as_ptr()) } == 0;
        }
        let skip = feedback.pcm.len().saturating_sub(2);
        if skip > 0 {
            self.upsampler = Upsampler::default();
        }
        for pcm in feedback.pcm.into_iter().skip(skip) {
            self.last_pcm = Instant::now();
            let pcm48 = self.upsampler.convert(&pcm, h_strength);
            if self.haptics_open
                && unsafe { ffi::rp_haptics_feed(pcm48.as_ptr(), pcm48.len()) } != 0
            {
                unsafe {
                    ffi::rp_haptics_close();
                }
                self.haptics_open = false;
            }
            if !self.haptics_open {
                self.rumble_is_haptics = true;
                self.last_rumble = crate::haptics::rumble(&pcm, h_strength);
                self.rumble_time = Instant::now();
                changed = true;
            }
        }
        if (self.rumble_is_haptics || h_strength == 0)
            && self.rumble_time.elapsed() > Duration::from_millis(80)
            && self.last_rumble != (0, 0)
        {
            self.last_rumble = (0, 0);
            changed = true;
        }
        if self.haptics_open && self.last_pcm.elapsed() > Duration::from_millis(200) {
            self.upsampler = Upsampler::default();
        }
        if self.last_rumble != (0, 0)
            && self.last_rumble_refresh.elapsed() > Duration::from_millis(50)
        {
            changed = true;
        }
        if changed {
            self.last_rumble_refresh = Instant::now();
            unsafe {
                ffi::rp_feedback(
                    self.last_rumble.0,
                    self.last_rumble.1,
                    if feedback.led.is_some() || refresh {
                        self.last_led
                            .as_ref()
                            .map_or(std::ptr::null(), |x| x.as_ptr())
                    } else {
                        std::ptr::null()
                    },
                    if feedback.triggers.is_some() || intensity_changed || refresh {
                        self.last_triggers
                            .as_ref()
                            .map_or(std::ptr::null(), |x| x.as_ptr())
                    } else {
                        std::ptr::null()
                    },
                    self.prefs.usb_effects as i32,
                    t_strength as i32,
                );
            }
        }
    }
    pub fn draw(&self) {
        unsafe {
            ffi::rp_begin(self.menu as i32);
        }
        let status = self
            .shared
            .as_ref()
            .map(|s| lock(&s.status).clone())
            .unwrap_or_else(|| "Disconnected".into());
        ffi::line(
            0,
            &format!(
                "PS5 Remote 0.2.0 | {} | RED/F1: menu | all TV capabilities UNVERIFIED",
                status
            ),
            false,
        );
        if let Some(s) = &self.shared {
            let q = lock(&s.video);
            ffi::line(1,&format!("Received:{} submitted:{} dropped:{} queue:{} feed_errors:{} RTT:{}ms presented_fps:UNKNOWN",s.received.load(Ordering::Relaxed),s.submitted.load(Ordering::Relaxed),q.dropped,q.frames.len(),s.feed_errors.load(Ordering::Relaxed),s.rtt_us.load(Ordering::Relaxed)/1000),false);
        }
        if !self.menu {
            unsafe {
                ffi::rp_present();
            }
            return;
        }
        ffi::line(2, &self.message, false);
        if let Some(e) = &self.editor {
            let value = match e.field {
                0 => self.host.clone(),
                1 => "*".repeat(self.account.len()),
                2 => "*".repeat(self.pin.len()),
                _ => "*".repeat(self.login.len()),
            };
            ffi::line(4, &format!("Editing field {}: {}", e.field, value), false);
            ffi::line(
                5,
                "Use arrows and OK. Physical keyboard typing also works. Back closes editor.",
                false,
            );
            for row in 0..((KEYS.len() + 2 + 9) / 10) {
                let mut text = String::new();
                for col in 0..10 {
                    let index = row * 10 + col;
                    if index >= KEYS.len() + 2 {
                        break;
                    }
                    let key = if index == KEYS.len() {
                        "DEL".into()
                    } else if index == KEYS.len() + 1 {
                        "DONE".into()
                    } else {
                        (KEYS.as_bytes()[index] as char).to_string()
                    };
                    text.push_str(&format!(
                        "{}{:^4}{} ",
                        if index == e.cursor { '[' } else { ' ' },
                        key,
                        if index == e.cursor { ']' } else { ' ' }
                    ));
                }
                ffi::line(7 + row as i32, &text, false);
            }
        } else {
            let rows = self.rows();
            for (i, row) in rows.iter().enumerate() {
                ffi::line(4 + i as i32, row, i == self.selected);
            }
            let mut buf = [0 as std::ffi::c_char; 256];
            unsafe {
                ffi::rp_diagnostics(buf.as_mut_ptr(), buf.len());
            }
            let bytes: Vec<u8> = buf
                .iter()
                .map(|x| *x as u8)
                .take_while(|x| *x != 0)
                .collect();
            ffi::line(31, &String::from_utf8_lossy(&bytes), false);
            ffi::line(33,"Volume/output: TV remote and TV settings. Effects default off; Bluetooth advanced effects unsupported.",false);
        }
        unsafe {
            ffi::rp_present();
        }
    }
    fn rows(&self) -> Vec<String> {
        let paired = self.paired.as_ref().is_some_and(|p| p.host == self.host);
        vec![
            format!("LAN IPv4: {}", self.host),
            format!(
                "Account ID (base64 of 8 bytes): {}",
                "*".repeat(self.account.len())
            ),
            format!("Link Device PIN: {}", "*".repeat(self.pin.len())),
            format!("PAIR / REPLACE PAIRING (currently paired: {})", paired),
            "CONNECT".into(),
            "DISCONNECT / CANCEL".into(),
            format!(
                "Codec: {}",
                ["H.264 SDR", "HEVC SDR", "HEVC HDR (experimental)"][self.prefs.codec as usize]
            ),
            format!("Resolution: {}p", self.prefs.height),
            format!("FPS: {}", self.prefs.fps),
            format!("Bitrate: {} kbps", self.prefs.bitrate),
            format!("Stick dead zone: {}", self.prefs.deadzone),
            format!("Audio jitter: {} ms", self.prefs.jitter_ms),
            format!("Reconnect (max 5): {}", self.prefs.reconnect),
            format!("Wake from rest: {}", self.prefs.wake),
            format!("USB triggers / LEDs: {}", self.prefs.usb_effects),
            format!("USB four-channel haptics: {}", self.prefs.usb_audio),
            format!(
                "Controller audio device: {}",
                if self.prefs.haptics_device.is_empty() {
                    "none"
                } else {
                    &self.prefs.haptics_device
                }
            ),
            format!("Haptics intensity: {}", self.prefs.haptics_strength),
            format!("Trigger intensity: {}", self.prefs.trigger_strength),
            format!("Mute: {}", self.muted),
            format!("Console login passcode: {}", "*".repeat(self.login.len())),
            "SUBMIT LOGIN PASSCODE".into(),
            "SAVE SETTINGS".into(),
            "TEST RUMBLE (API result is not physical confirmation)".into(),
            "RETURN TO STREAM".into(),
            "QUIT".into(),
        ]
    }
    pub fn log_stats(&self) {
        if self.worker.is_none() {
            return;
        }
        if let Some(s) = &self.shared {
            let v = lock(&s.video);
            let a = lock(&s.audio);
            eprintln!("Stats codec={} network_fps={} network_lost={} received={} submitted={} dropped={} video_queue={} feed_errors={} audio_queue={} audio_lost={} audio_errors={} rtt_us={} presented_fps=UNKNOWN",self.active_codec,s.network_fps.load(Ordering::Relaxed),s.network_lost.load(Ordering::Relaxed),s.received.load(Ordering::Relaxed),s.submitted.load(Ordering::Relaxed),v.dropped,v.frames.len(),s.feed_errors.load(Ordering::Relaxed),a.packets.len(),a.lost,s.audio_errors.load(Ordering::Relaxed),s.rtt_us.load(Ordering::Relaxed));
        }
    }
    pub fn shutdown(&mut self) {
        self.disconnect();
        let until = Instant::now() + Duration::from_secs(2);
        while self.worker.as_ref().is_some_and(|w| !w.is_finished()) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            let _ = self.worker.take().unwrap().join();
        }
        self.clear_outputs();
    }
}
fn weaker(user: u8, console: u8) -> u8 {
    if user == 0 || console == 0 {
        0
    } else {
        user.max(console).min(3)
    }
}
fn scale_rumble((l, r): (u8, u8), s: u8) -> (u8, u8) {
    let scale = match s {
        0 => 0,
        1 => 100,
        2 => 66,
        _ => 33,
    };
    (
        (l as u16 * scale / 100) as u8,
        (r as u16 * scale / 100) as u8,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intensity_respects_off_and_weaker_setting() {
        assert_eq!(weaker(0, 1), 0);
        assert_eq!(weaker(1, 3), 3);
        assert_eq!(weaker(2, 1), 2);
        assert_eq!(scale_rumble((255, 255), 0), (0, 0));
    }

    fn app() -> (App, std::path::PathBuf) {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "psremote-ui-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let store = Store::open(path.clone()).unwrap();
        (App::new(store, Preferences::default(), None), path)
    }
    #[test]
    fn setup_editor_saves_host_and_never_draws_secrets() {
        let (mut a, path) = app();
        a.event(&Event {
            kind: 5,
            ..Default::default()
        });
        let mut input = Event {
            kind: 7,
            ..Default::default()
        };
        input.text[..12].copy_from_slice(b"192.168.1.10");
        a.event(&input);
        a.event(&Event {
            kind: 6,
            ..Default::default()
        });
        a.selected = 22;
        a.event(&Event {
            kind: 5,
            ..Default::default()
        });
        assert_eq!(a.store.preferences().unwrap().host, "192.168.1.10");
        a.account = "AQIDBAUGBwg=".into();
        a.pin = "98765432".into();
        a.login = "4321".into();
        a.draw();
        let drawn = crate::native_test::LINES.with(|s| s.borrow().join("\n"));
        for secret in [&a.account, &a.pin, &a.login] {
            assert!(!drawn.contains(secret));
        }
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn cancelled_pairing_result_does_not_replace_existing_credentials() {
        let (mut a, path) = app();
        let old = Pairing {
            version: 1,
            host: "192.168.1.10".into(),
            regist_key: "31".repeat(16),
            rp_key: "01".repeat(16),
        };
        a.store.save_pairing(&old).unwrap();
        a.paired = Some(old.clone());
        let (tx, rx) = mpsc::channel();
        a.pair_rx = Some(rx);
        a.disconnect();
        let mut replacement = old.clone();
        replacement.rp_key = "02".repeat(16);
        tx.send(Ok(replacement)).unwrap();
        a.update();
        assert_eq!(a.store.pairing().unwrap().unwrap().rp_key, old.rp_key);
        assert_eq!(a.paired.as_ref().unwrap().rp_key, old.rp_key);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn opening_menu_releases_game_input_and_disconnect_cancels_retry() {
        let (mut a, path) = app();
        let s = Shared::new(&a.prefs);
        a.shared = Some(s.clone());
        a.menu = false;
        let mut e = Event::default();
        e.input_changed = 1;
        e.input.connected = 1;
        e.input.buttons = 1;
        a.event(&e);
        assert_eq!(lock(&s.inputs).pop().unwrap().buttons, 1);
        a.event(&Event {
            kind: 11,
            ..Default::default()
        });
        assert_eq!(lock(&s.inputs).pop().unwrap().buttons, 0);
        a.retry_at = Some(Instant::now());
        a.disconnect();
        assert!(a.retry_at.is_none());
        assert!(s.stop.load(Ordering::Acquire));
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn passcode_prompt_focus_does_not_prevent_selecting_submit() {
        let (mut a, path) = app();
        let shared = Shared::new(&a.prefs);
        shared.passcode_requested.store(true, Ordering::Release);
        a.shared = Some(shared.clone());
        a.update();
        assert_eq!(a.selected, 20);
        a.event(&Event {
            kind: 2,
            ..Default::default()
        });
        a.update();
        assert_eq!(a.selected, 21);
        a.login = "1234".into();
        a.event(&Event {
            kind: 5,
            ..Default::default()
        });
        assert!(matches!(
            lock(&shared.passcode).take(),
            Some(Passcode::Digits(_))
        ));
        assert!(a.login.is_empty());
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn disabling_reconnect_cancels_an_already_scheduled_attempt() {
        let (mut a, path) = app();
        a.retry_at = Some(Instant::now());
        a.fallback = Some(0);
        a.selected = 12;
        a.event(&Event {
            kind: 5,
            ..Default::default()
        });
        assert!(!a.prefs.reconnect);
        assert!(a.retry_at.is_none());
        assert!(a.fallback.is_none());
        std::fs::remove_dir_all(path).unwrap();
    }
}
