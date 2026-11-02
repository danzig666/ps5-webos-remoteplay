// SPDX-License-Identifier: AGPL-3.0-only
use psremote::rp_stream::{AudioHeader, ControllerState};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};
pub const VIDEO_LIMIT: usize = 3;
pub fn nalus(data: &[u8]) -> Vec<&[u8]> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = if data[i..].starts_with(&[0, 0, 0, 1]) {
            4
        } else if data[i..].starts_with(&[0, 0, 1]) {
            3
        } else {
            i += 1;
            continue;
        };
        starts.push((i, n));
        i += n;
    }
    starts
        .iter()
        .enumerate()
        .filter_map(|(k, (p, n))| {
            let end = starts.get(k + 1).map_or(data.len(), |x| x.0);
            if p + n < end {
                Some(&data[p + n..end])
            } else {
                None
            }
        })
        .collect()
}
pub fn keyframe(data: &[u8], hevc: bool) -> bool {
    nalus(data).iter().any(|n| {
        if hevc {
            (16..=21).contains(&((n[0] >> 1) & 63))
        } else {
            n[0] & 31 == 5
        }
    })
}
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub key: bool,
    pub at: Instant,
    pub width: u32,
    pub height: u32,
}
pub struct VideoQueue {
    pub frames: VecDeque<VideoFrame>,
    pub header: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub dropped: u64,
    pub waiting: bool,
    pub hevc: bool,
}
impl VideoQueue {
    pub fn new(hevc: bool, width: u32, height: u32) -> Self {
        Self {
            frames: VecDeque::new(),
            header: Vec::new(),
            width,
            height,
            dropped: 0,
            waiting: true,
            hevc,
        }
    }
    pub fn reset(&mut self) {
        self.dropped += self.frames.len() as u64;
        self.frames.clear();
        self.waiting = true;
    }
    pub fn header(&mut self, h: &[u8]) {
        self.reset();
        if h.len() <= 65536 {
            self.header = h.to_vec();
        } else {
            self.header.clear();
        }
    }
    pub fn push(&mut self, data: &[u8], now: Instant) -> bool {
        if data.len() > 2 * 1024 * 1024 {
            self.reset();
            self.dropped += 1;
            return false;
        }
        if self.frames.len() >= VIDEO_LIMIT {
            self.reset();
        }
        let key = keyframe(data, self.hevc);
        if (self.waiting && !key) || self.header.is_empty() {
            self.dropped += 1;
            return false;
        }
        let mut au = Vec::with_capacity(data.len() + self.header.len());
        if key {
            au.extend_from_slice(&self.header);
        }
        au.extend_from_slice(data);
        self.waiting = false;
        self.frames.push_back(VideoFrame {
            data: au,
            key,
            at: now,
            width: self.width,
            height: self.height,
        });
        true
    }
    pub fn pop(&mut self, now: Instant) -> Option<VideoFrame> {
        if self
            .frames
            .front()
            .is_some_and(|f| now.duration_since(f.at) > Duration::from_millis(100))
        {
            self.reset();
            return None;
        }
        self.frames.pop_front()
    }
}
pub struct AudioQueue {
    pub header: Option<AudioHeader>,
    pub packets: HashMap<u16, Vec<u8>>,
    next: Option<u16>,
    since: Instant,
    pub lost: u64,
    pub jitter: Duration,
    pub last_feed: Option<Instant>,
}
impl AudioQueue {
    pub fn new(jitter: u64) -> Self {
        Self {
            header: None,
            packets: HashMap::new(),
            next: None,
            since: Instant::now(),
            lost: 0,
            jitter: Duration::from_millis(jitter),
            last_feed: None,
        }
    }
    pub fn push(&mut self, seq: u16, data: &[u8], now: Instant) {
        if data.is_empty() || data.len() > 4096 {
            return;
        }
        if let Some(n) = self.next {
            let delta = seq.wrapping_sub(n) as i16;
            if delta < 0 {
                if self.last_feed.is_none() && delta >= -24 {
                    self.next = Some(seq);
                } else {
                    return;
                }
            }
        }
        if self.packets.len() >= 24 {
            self.lost += self.packets.len() as u64;
            self.packets.clear();
            self.next = None;
            self.last_feed = None;
        }
        if self.next.is_none() {
            self.next = Some(seq);
            self.since = now;
        }
        self.packets.entry(seq).or_insert_with(|| data.to_vec());
    }
    pub fn pop(&mut self, now: Instant) -> Option<Vec<u8>> {
        if now.duration_since(self.since) < self.jitter && self.last_feed.is_none() {
            return None;
        }
        let step = self.header.map_or(Duration::from_millis(10), |h| {
            Duration::from_secs_f64(h.frame_size as f64 / h.rate.max(1) as f64)
        });
        if self.last_feed.is_some_and(|t| now.duration_since(t) < step) {
            return None;
        }
        let mut n = self.next?;
        if !self.packets.contains_key(&n) {
            if now.duration_since(self.since) < self.jitter {
                return None;
            }
            let first = *self.packets.keys().min_by_key(|x| x.wrapping_sub(n))?;
            self.lost += first.wrapping_sub(n) as u64;
            n = first;
        }
        let data = self.packets.remove(&n)?;
        self.next = Some(n.wrapping_add(1));
        self.since = now;
        self.last_feed = Some(self.last_feed.map_or(now, |t| {
            if now.duration_since(t) > Duration::from_millis(100) {
                now
            } else {
                t + step
            }
        }));
        Some(data)
    }
}
pub struct Inputs {
    states: VecDeque<ControllerState>,
    pub overflow: u64,
}
impl Default for Inputs {
    fn default() -> Self {
        Self {
            states: VecDeque::new(),
            overflow: 0,
        }
    }
}
impl Inputs {
    pub fn push(&mut self, s: ControllerState) {
        if self.states.back() == Some(&s) {
            return;
        }
        if let Some(last) = self.states.back_mut() {
            if last.buttons == s.buttons
                && last.touch.is_some() == s.touch.is_some()
                && (last.l2 == 0) == (s.l2 == 0)
                && (last.r2 == 0) == (s.r2 == 0)
            {
                *last = s;
                return;
            }
        }
        if self.states.len() >= 256 {
            self.states.clear();
            self.states.push_back(ControllerState::default());
            self.overflow += 1;
        }
        self.states.push_back(s);
    }
    pub fn pop(&mut self) -> Option<ControllerState> {
        self.states.pop_front()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overflow_discards_dependencies_until_idr() {
        let now = Instant::now();
        let mut q = VideoQueue::new(false, 1920, 1080);
        q.header(&[0, 0, 1, 7, 1]);
        assert!(!q.push(&[0, 0, 1, 1], now));
        assert!(q.push(&[0, 0, 1, 5], now));
        for _ in 0..2 {
            assert!(q.push(&[0, 0, 1, 1], now));
        }
        assert!(!q.push(&[0, 0, 1, 1], now));
        assert!(q.frames.is_empty());
        assert!(q.push(&[0, 0, 1, 5], now));
        assert!(q.pop(now).unwrap().data.starts_with(&[0, 0, 1, 7]));
    }
    #[test]
    fn short_tap_and_disconnect_are_ordered() {
        let mut q = Inputs::default();
        let mut s = ControllerState::default();
        s.buttons = 1;
        q.push(s);
        q.push(ControllerState::default());
        assert_eq!(q.pop().unwrap().buttons, 1);
        assert_eq!(q.pop().unwrap().buttons, 0);
    }
    #[test]
    fn jitter_reorders_across_wrap() {
        let mut q = AudioQueue::new(10);
        let now = Instant::now();
        q.push(65535, &[1], now);
        q.push(1, &[3], now);
        q.push(0, &[2], now);
        q.since = now - Duration::from_millis(20);
        let a = q.pop(now).unwrap();
        assert_eq!(a, [1]);
        assert_eq!(q.pop(now + Duration::from_millis(10)).unwrap(), [2]);
        assert_eq!(q.pop(now + Duration::from_millis(20)).unwrap(), [3]);
    }
    #[test]
    fn hevc_irap() {
        assert!(keyframe(&[0, 0, 0, 1, 19 << 1, 1], true));
        assert!(!keyframe(&[0, 0, 1, 1 << 1, 1], true));
    }
}

#[cfg(test)]
mod soak {
    use super::*;
    /// Two hours of virtual 60 fps time; intentionally slower consumer and periodic stalls.
    /// This checks bounded state/recovery only, never real-time performance or TV playback.
    #[test]
    fn simulated_two_hour_queue_soak() {
        let start = Instant::now();
        let mut q = VideoQueue::new(false, 1920, 1080);
        q.header(&[0, 0, 1, 7, 1, 0, 0, 1, 8, 1]);
        let mut input = Inputs::default();
        let mut audio = AudioQueue::new(20);
        for frame in 0..432000u64 {
            let now = start + Duration::from_nanos(frame * 1_000_000_000 / 60);
            let nal = if frame % 60 == 0 { 5 } else { 1 };
            q.push(&[0, 0, 1, nal, 0x80], now);
            if frame % 2 == 0 && frame % 600 > 30 {
                let _ = q.pop(now);
            }
            let mut state = ControllerState::default();
            state.buttons = (frame % 2) as u32;
            input.push(state);
            if frame % 3 == 0 {
                let _ = input.pop();
            }
            audio.push(frame as u16, &[0xf8, 0xff, 0xfe], now);
            if frame % 2 == 0 {
                let _ = audio.pop(now);
            }
            assert!(q.frames.len() <= 3);
            assert!(audio.packets.len() <= 24);
            assert!(input.states.len() <= 256);
        }
        assert!(q.dropped > 0);
        assert!(input.overflow > 0);
        let now = start + Duration::from_secs(7201);
        q.reset();
        assert!(q.push(&[0, 0, 1, 5, 0x80], now));
        assert!(q.pop(now).unwrap().key);
    }
    #[test]
    fn audio_pacing_does_not_throttle_100hz_to_60hz_ui() {
        let start = Instant::now();
        let mut q = AudioQueue::new(10);
        q.header = Some(AudioHeader {
            channels: 2,
            bits: 16,
            rate: 48000,
            frame_size: 480,
        });
        for seq in 0..10 {
            q.push(seq, &[1], start);
        }
        assert!(q.pop(start + Duration::from_millis(20)).is_some());
        let later = start + Duration::from_millis(37);
        assert!(q.pop(later).is_some());
        assert!(q.pop(later).is_none());
        let later = start + Duration::from_millis(54);
        assert!(q.pop(later).is_some());
        assert!(q.pop(later).is_some());
    }
}
