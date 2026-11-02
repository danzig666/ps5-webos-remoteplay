// SPDX-License-Identifier: AGPL-3.0-only
//! PS5Upload pad_haptics.rs upsampling algorithm, adapted for bounded SDL queued audio.
//! 3 kHz stereo s16le -> 48 kHz four-channel s16le. Speaker channels stay silent.
#[derive(Default)]
pub struct Upsampler {
    last: [f32; 2],
}
impl Upsampler {
    pub fn convert(&mut self, pcm: &[u8], strength: u8) -> Vec<i16> {
        if pcm.len() != 120 {
            return Vec::new();
        } // one 10 ms stereo frame
        let gain = match strength {
            0 => 0.0,
            1 => 1.0,
            2 => 0.66,
            _ => 0.33,
        };
        let mut out = Vec::with_capacity(1920);
        for b in pcm.chunks_exact(4) {
            let next = [
                i16::from_le_bytes([b[0], b[1]]) as f32,
                i16::from_le_bytes([b[2], b[3]]) as f32,
            ];
            for k in 1..=16 {
                let t = k as f32 / 16.0;
                out.extend_from_slice(&[
                    0,
                    0,
                    ((self.last[0] + (next[0] - self.last[0]) * t) * gain) as i16,
                    ((self.last[1] + (next[1] - self.last[1]) * t) * gain) as i16,
                ]);
            }
            self.last = next;
        }
        out
    }
}
pub fn rumble(pcm: &[u8], strength: u8) -> (u8, u8) {
    if strength == 0 {
        return (0, 0);
    }
    let mut sums = [0.0f64; 2];
    let mut n = 0;
    for b in pcm.chunks_exact(4) {
        for i in 0..2 {
            let v = i16::from_le_bytes([b[2 * i], b[2 * i + 1]]) as f64 / 32768.0;
            sums[i] += v * v;
        }
        n += 1;
    }
    if n == 0 {
        return (0, 0);
    }
    let gain = match strength {
        1 => 1.0,
        2 => 0.66,
        _ => 0.33,
    };
    (
        ((sums[0] / n as f64).sqrt() * 255.0 * gain).min(255.0) as u8,
        ((sums[1] / n as f64).sqrt() * 255.0 * gain).min(255.0) as u8,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channel_routing_and_off() {
        let pcm = [0xff, 0x7f, 0, 0].repeat(30);
        let mut u = Upsampler::default();
        let out = u.convert(&pcm, 1);
        assert_eq!(out.len(), 1920);
        for f in out.chunks_exact(4) {
            assert_eq!(f[0], 0);
            assert_eq!(f[1], 0);
            assert_eq!(f[3], 0);
        }
        assert_eq!(out[1918], 32767);
        assert!(u.convert(&pcm, 0).iter().all(|x| *x == 0));
        assert_eq!(rumble(&pcm, 0), (0, 0));
        assert!(rumble(&pcm, 1).0 > 250);
    }
}
