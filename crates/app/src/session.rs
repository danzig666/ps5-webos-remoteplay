// SPDX-License-Identifier: AGPL-3.0-only
use crate::{
    config::{Pairing, Preferences},
    hdr::{self, HdrMetadata},
    queues::{AudioQueue, Inputs, VideoQueue},
};
use psremote::rp_stream::{
    self, AudioHeader, ControllerState, Passcode, Progress, StreamSink, TriggerEffect,
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
#[derive(Clone, Default)]
pub struct Feedback {
    pub rumble: Option<(u8, u8)>,
    pub led: Option<[u8; 3]>,
    pub triggers: Option<(TriggerEffect, TriggerEffect)>,
    pub intensity: (u8, u8),
    pub pcm: VecDeque<Vec<u8>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Stopped,
    Transient,
    Refused,
    Panic,
}
pub struct Shared {
    pub stop: AtomicBool,
    pub keyframe: AtomicBool,
    pub video: Mutex<VideoQueue>,
    pub audio: Mutex<AudioQueue>,
    pub inputs: Mutex<Inputs>,
    pub feedback: Mutex<Feedback>,
    pub status: Mutex<String>,
    pub outcome: Mutex<Outcome>,
    pub passcode_requested: AtomicBool,
    pub passcode: Mutex<Option<Passcode>>,
    pub metadata: Mutex<Option<HdrMetadata>>,
    pub network_fps: AtomicU64,
    pub network_lost: AtomicU64,
    pub received: AtomicU64,
    pub submitted: AtomicU64,
    pub feed_errors: AtomicU64,
    pub audio_errors: AtomicU64,
    pub rtt_us: AtomicU64,
}
impl Shared {
    pub fn new(p: &Preferences) -> Arc<Self> {
        Arc::new(Self {
            stop: AtomicBool::new(false),
            keyframe: AtomicBool::new(true),
            video: Mutex::new(VideoQueue::new(
                p.codec != 0,
                if p.height == 720 { 1280 } else { 1920 },
                p.height,
            )),
            audio: Mutex::new(AudioQueue::new(p.jitter_ms)),
            inputs: Mutex::new(Inputs::default()),
            feedback: Mutex::new(Feedback {
                intensity: (1, 1),
                ..Default::default()
            }),
            status: Mutex::new("Connecting".into()),
            outcome: Mutex::new(Outcome::Running),
            passcode_requested: AtomicBool::new(false),
            passcode: Mutex::new(None),
            metadata: Mutex::new(None),
            network_fps: AtomicU64::new(0),
            network_lost: AtomicU64::new(0),
            received: AtomicU64::new(0),
            submitted: AtomicU64::new(0),
            feed_errors: AtomicU64::new(0),
            audio_errors: AtomicU64::new(0),
            rtt_us: AtomicU64::new(0),
        })
    }
    pub fn cancel(&self) {
        lock(&self.inputs).release();
        self.stop.store(true, Ordering::Release);
        *lock(&self.passcode) = Some(Passcode::Cancel);
    }
}
struct Sink {
    shared: Arc<Shared>,
}
impl StreamSink for Sink {
    fn video_format(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 || width > 1920 || height > 1080 {
            self.shared.cancel();
            return;
        }
        let mut q = lock(&self.shared.video);
        q.width = width;
        q.height = height;
        q.reset();
        self.shared.keyframe.store(true, Ordering::Relaxed);
    }
    fn video(&mut self, data: &[u8], header: bool) -> bool {
        if self.shared.stop.load(Ordering::Acquire) {
            return false;
        }
        if let Some(m) = hdr::parse(data) {
            *lock(&self.shared.metadata) = Some(m);
        }
        let mut q = lock(&self.shared.video);
        if header {
            q.header(data);
        } else {
            self.shared.received.fetch_add(1, Ordering::Relaxed);
            if !q.push(data, Instant::now()) {
                self.shared.keyframe.store(true, Ordering::Relaxed);
            }
        }
        true
    }
    fn audio(&mut self, frame: &[u8], header: bool) -> bool {
        let mut q = lock(&self.shared.audio);
        if header {
            q.header = AudioHeader::parse(frame).filter(|h| {
                [1, 2].contains(&h.channels)
                    && [24000, 48000].contains(&h.rate)
                    && (120..=1920).contains(&h.frame_size)
            });
        } else if frame.len() > 2 {
            q.push(
                u16::from_be_bytes([frame[0], frame[1]]),
                &frame[2..],
                Instant::now(),
            );
        }
        !self.shared.stop.load(Ordering::Acquire)
    }
    fn input(&mut self) -> Option<ControllerState> {
        lock(&self.shared.inputs).pop()
    }
    fn rumble(&mut self, l: u8, r: u8) {
        lock(&self.shared.feedback).rumble = Some((l, r));
    }
    fn light_bar(&mut self, rgb: [u8; 3]) {
        lock(&self.shared.feedback).led = Some(rgb);
    }
    fn trigger_effects(&mut self, l: TriggerEffect, r: TriggerEffect) {
        lock(&self.shared.feedback).triggers = Some((l, r));
    }
    fn dualsense_intensity(&mut self, h: u8, t: u8) {
        lock(&self.shared.feedback).intensity = (h.min(3), t.min(3));
    }
    fn haptics(&mut self, pcm: &[u8]) {
        let mut f = lock(&self.shared.feedback);
        if f.pcm.len() >= 8 {
            f.pcm.pop_front();
        }
        if pcm.len() == 120 {
            f.pcm.push_back(pcm.to_vec());
        }
    }
    fn progress(&mut self, p: Progress) {
        let text = match p {
            Progress::Waking => "Waking console",
            Progress::Nonce => "Authenticating",
            Progress::SessionId => "Opening transport",
            Progress::TakionConnected => "Transport connected",
            Progress::Bang => "Keys established",
            Progress::StreamInfo(_) => "Starting decoder",
            Progress::FirstFrame(_) => "Receiving video",
            Progress::Stats(s) => {
                self.shared.network_fps.store(s.frames, Ordering::Relaxed);
                self.shared
                    .network_lost
                    .fetch_add(s.frames_lost, Ordering::Relaxed);
                if let Some(rtt) = s.console_rtt_ms {
                    self.shared
                        .rtt_us
                        .store((rtt * 1000.0) as u64, Ordering::Relaxed);
                }
                return;
            }
        };
        *lock(&self.shared.status) = text.into();
    }
    fn stopped(&mut self) -> bool {
        self.shared.stop.load(Ordering::Acquire)
    }
    fn wants_keyframe(&mut self) -> bool {
        self.shared.keyframe.swap(false, Ordering::Relaxed)
    }
    fn passcode_needed(&mut self, retry: bool) {
        *lock(&self.shared.passcode) = None;
        self.shared
            .passcode_requested
            .store(true, Ordering::Release);
        *lock(&self.shared.status) = if retry {
            "Passcode incorrect; enter again"
        } else {
            "Console login passcode required"
        }
        .into();
    }
    fn passcode(&mut self) -> Option<Passcode> {
        if self.stopped() {
            Some(Passcode::Cancel)
        } else {
            lock(&self.shared.passcode).take()
        }
    }
}
pub fn start(
    pair: Pairing,
    prefs: Preferences,
    dualsense: bool,
    shared: Arc<Shared>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let creds = pair.creds().map_err(|_| Outcome::Refused)?;
            let cfg = rp_stream::StreamConfig {
                width: if prefs.height == 720 { 1280 } else { 1920 },
                height: prefs.height,
                fps: prefs.fps,
                bitrate_kbps: prefs.bitrate,
                codec: match prefs.codec {
                    1 => rp_stream::Codec::H265,
                    2 => rp_stream::Codec::H265Hdr,
                    _ => rp_stream::Codec::H264,
                },
                wake: prefs.wake,
                mic: false,
                keyboard: false,
                dualsense,
            };
            rp_stream::run_stream(
                &pair.host,
                &creds,
                cfg,
                Duration::from_secs(24 * 3600),
                &mut Sink {
                    shared: shared.clone(),
                },
            )
            .map(|_| Outcome::Stopped)
            .map_err(|e| {
                // Never surface Debug/Display of protocol errors: upstream messages may embed secrets.
                if e.downcast_ref::<rp_stream::guard::NotReady>().is_some()
                    || e.chain()
                        .any(|e| e.downcast_ref::<std::io::Error>().is_some())
                {
                    Outcome::Transient
                } else {
                    Outcome::Refused
                }
            })
        }));
        let outcome = if shared.stop.load(Ordering::Acquire) {
            Outcome::Stopped
        } else {
            match result {
                Ok(Ok(o)) => o,
                Ok(Err(o)) => o,
                Err(_) => Outcome::Panic,
            }
        };
        *lock(&shared.outcome) = outcome;
        *lock(&shared.status) = match outcome {
            Outcome::Stopped => "Disconnected",
            Outcome::Transient => "LAN unavailable; retry possible",
            Outcome::Refused => "Session refused; check pairing / console",
            Outcome::Panic => "Session worker failed",
            Outcome::Running => "Connecting",
        }
        .into();
    })
}
