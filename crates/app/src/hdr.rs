// SPDX-License-Identifier: AGPL-3.0-only
use crate::queues::nalus;
/// ST2086 primaries are G,B,R in the HEVC SEI. Keep the exact received integers.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct HdrMetadata {
    pub gx: i32,
    pub gy: i32,
    pub bx: i32,
    pub by: i32,
    pub rx: i32,
    pub ry: i32,
    pub wx: i32,
    pub wy: i32,
    pub max_luma: i32,
    pub min_luma: i32,
    pub max_cll: i32,
    pub max_fall: i32,
}
pub fn parse(data: &[u8]) -> Option<HdrMetadata> {
    let mut metadata = None;
    let mut cll = (0, 0);
    for nal in nalus(data) {
        if nal.len() < 3 || ![39, 40].contains(&((nal[0] >> 1) & 63)) {
            continue;
        }
        let mut rbsp = Vec::new();
        let mut zeros = 0;
        for &b in &nal[2..] {
            if zeros >= 2 && b == 3 {
                zeros = 0;
                continue;
            }
            rbsp.push(b);
            zeros = if b == 0 { zeros + 1 } else { 0 };
        }
        let mut pos = 0;
        while pos + 2 <= rbsp.len() {
            let mut kind = 0usize;
            while pos < rbsp.len() && rbsp[pos] == 255 {
                kind += 255;
                pos += 1;
            }
            if pos >= rbsp.len() {
                break;
            }
            kind += rbsp[pos] as usize;
            pos += 1;
            let mut len = 0usize;
            while pos < rbsp.len() && rbsp[pos] == 255 {
                len += 255;
                pos += 1;
            }
            if pos >= rbsp.len() {
                break;
            }
            len += rbsp[pos] as usize;
            pos += 1;
            if len > rbsp.len() - pos {
                break;
            }
            let b = &rbsp[pos..pos + len];
            pos += len;
            let u16at = |n| u16::from_be_bytes([b[n], b[n + 1]]) as i32;
            if kind == 137 && len == 24 {
                let max = u32::from_be_bytes(b[16..20].try_into().ok()?);
                let min = u32::from_be_bytes(b[20..24].try_into().ok()?);
                if max > i32::MAX as u32 || min > max {
                    continue;
                }
                metadata = Some(HdrMetadata {
                    gx: u16at(0),
                    gy: u16at(2),
                    bx: u16at(4),
                    by: u16at(6),
                    rx: u16at(8),
                    ry: u16at(10),
                    wx: u16at(12),
                    wy: u16at(14),
                    max_luma: max as i32,
                    min_luma: min as i32,
                    ..Default::default()
                });
            } else if kind == 144 && len == 4 {
                cll = (u16at(0), u16at(2));
            }
        }
    }
    metadata.map(|mut m| {
        m.max_cll = cll.0;
        m.max_fall = cll.1;
        m
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn received_mastering_metadata_and_truncation() {
        let mut payload = vec![137, 24];
        for v in 1..=8u16 {
            payload.extend_from_slice(&v.to_be_bytes());
        }
        payload.extend_from_slice(&10000u32.to_be_bytes());
        payload.extend_from_slice(&1u32.to_be_bytes());
        let mut b = vec![0, 0, 1, 39 << 1, 1];
        let mut zeros = 0;
        for c in payload {
            if zeros >= 2 && c <= 3 {
                b.push(3);
                zeros = 0;
            }
            b.push(c);
            zeros = if c == 0 { zeros + 1 } else { 0 };
        }
        let m = parse(&b).unwrap();
        assert_eq!(m.gx, 1);
        assert_eq!(m.rx, 5);
        assert_eq!(m.max_luma, 10000);
        b.pop();
        assert!(parse(&b).is_none());
    }
}
