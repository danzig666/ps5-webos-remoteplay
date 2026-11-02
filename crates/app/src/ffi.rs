// SPDX-License-Identifier: AGPL-3.0-only
use crate::{config::Preferences, hdr::HdrMetadata};
use psremote::rp_stream::{feedback::MotionSample, ControllerState};
use std::ffi::{c_char, CString};
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct RawInput {
    pub buttons: u32,
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
    pub l2: i16,
    pub r2: i16,
    pub touch_x: u16,
    pub touch_y: u16,
    pub touch_active: u32,
    pub connected: u32,
    pub motion: u32,
    pub timestamp: u32,
    pub gyro: [i32; 3],
    pub accel: [i32; 3],
}
#[repr(C)]
pub struct Event {
    pub kind: i32,
    pub value: i32,
    pub input_changed: i32,
    pub text: [u8; 64],
    pub input: RawInput,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            kind: 0,
            value: 0,
            input_changed: 0,
            text: [0; 64],
            input: RawInput::default(),
        }
    }
}
impl RawInput {
    pub fn normalized(&self, p: &Preferences) -> ControllerState {
        if self.connected == 0 {
            return ControllerState::default();
        }
        let mut bits = 0;
        for (s, t) in [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 13),
            (5, 15),
            (6, 12),
            (7, 10),
            (8, 11),
            (9, 8),
            (10, 9),
            (11, 6),
            (12, 7),
            (13, 4),
            (14, 5),
            (20, 14),
        ] {
            if self.buttons & (1u32 << s) != 0 {
                bits |= 1u32 << t;
            }
        }
        let axis = |v: i16| {
            if (v as i32).abs() < (p.deadzone as i32) {
                0
            } else {
                v
            }
        };
        let trigger = |v: i16| ((v.max(0) as u32) * 255 / 32767) as u8;
        ControllerState {
            buttons: bits,
            lx: axis(self.lx),
            ly: axis(self.ly),
            rx: axis(self.rx),
            ry: axis(self.ry),
            l2: trigger(self.l2),
            r2: trigger(self.r2),
            touch: (self.touch_active != 0)
                .then_some([self.touch_x.min(1919), self.touch_y.min(941)]),
            motion: (self.motion != 0).then_some(MotionSample {
                gyro: self.gyro,
                accel: self.accel,
                t: self.timestamp,
            }),
        }
    }
}
extern "C" {
    pub fn rp_open() -> i32;
    pub fn rp_poll(event: *mut Event) -> i32;
    pub fn rp_begin(menu: i32);
    pub fn rp_text(row: i32, text: *const c_char, selected: i32);
    pub fn rp_present();
    pub fn rp_close();
    pub fn rp_media_open(
        width: i32,
        height: i32,
        fps: i32,
        codec: i32,
        rate: i32,
        channels: i32,
        samples: i32,
        hdr: *const HdrMetadata,
    ) -> i32;
    pub fn rp_video(data: *const u8, size: usize, key: i32) -> i32;
    pub fn rp_audio(data: *const u8, size: usize, muted: i32) -> i32;
    pub fn rp_media_close();
    pub fn rp_feedback(
        left: u8,
        right: u8,
        rgb: *const u8,
        triggers: *const u8,
        enabled: i32,
        strength: i32,
    );
    pub fn rp_effects_reset();
    pub fn rp_haptics_open(name: *const c_char) -> i32;
    pub fn rp_haptics_feed(pcm: *const i16, samples: usize) -> i32;
    pub fn rp_haptics_close();
    pub fn rp_audio_device_count() -> i32;
    pub fn rp_audio_device_name(index: i32, out: *mut c_char, size: usize);
    pub fn rp_diagnostics(out: *mut c_char, size: usize) -> i32;
}
pub fn line(row: i32, s: &str, selected: bool) {
    let clean: String = s
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .take(152)
        .collect();
    let c = CString::new(clean).unwrap();
    unsafe {
        rp_text(row, c.as_ptr(), selected as i32);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axes_buttons_and_touch_mapping() {
        let mut r = RawInput::default();
        r.connected = 1;
        r.buttons = (1 << 11) | (1 << 20) | (1 << 4);
        r.lx = i16::MIN;
        r.rx = 100;
        r.l2 = i16::MAX;
        r.touch_active = 1;
        r.touch_x = 65535;
        let s = r.normalized(&Preferences::default());
        assert_eq!(s.buttons, (1 << 6) | (1 << 14) | (1 << 13));
        assert_eq!(s.lx, i16::MIN);
        assert_eq!(s.rx, 0);
        assert_eq!(s.l2, 255);
        assert_eq!(s.touch, Some([1919, 0]));
        r.connected = 0;
        assert_eq!(
            r.normalized(&Preferences::default()),
            ControllerState::default()
        );
    }
    #[test]
    fn c_layout() {
        assert_eq!(std::mem::size_of::<RawInput>(), 60);
        assert_eq!(std::mem::size_of::<Event>(), 136);
    }
}
