//! M0 only: bounded local H.264 fixture playback, no network or console code.
use std::time::{Duration, Instant};

/// Split an AUD-delimited Annex-B fixture without rewriting its parameter sets.
/// The encoder must use aud=1, repeat-headers=1 and no B frames.
pub fn access_units(data: &[u8]) -> Result<Vec<(&[u8], bool)>, &'static str> {
    if data.len() > 32 * 1024 * 1024 { return Err("fixture exceeds 32 MiB"); }
    let mut aud = Vec::new();
    let mut idrs = Vec::new();
    let mut i = 0;
    while i + 3 < data.len() {
        let n = if data[i..].starts_with(&[0,0,0,1]) {4}
            else if data[i..].starts_with(&[0,0,1]) {3} else {i+=1; continue};
        if i+n == data.len() { return Err("truncated NAL"); }
        match data[i+n] & 31 {9 => aud.push(i), 5 => idrs.push(i), _ => ()}
        i += n+1;
    }
    if aud.is_empty() || aud[0] != 0 || idrs.is_empty() { return Err("need AUD-delimited H.264 with IDR"); }
    aud.push(data.len());
    Ok(aud.windows(2).map(|w| (&data[w[0]..w[1]], idrs.iter().any(|p| *p >= w[0] && *p < w[1]))).collect())
}

extern "C" {
    fn platform_open() -> i32;
    fn platform_tick() -> i32;
    fn platform_feed(data: *const u8, size: usize, key: i32) -> i32;
    fn platform_close();
}

fn run() -> Result<(), String> {
    let data = std::fs::read("assets/test.h264").map_err(|e| e.to_string())?;
    let frames = access_units(&data).map_err(str::to_string)?;
    eprintln!("M0 fixture units={} bytes={} submitted != presented; visual verification required", frames.len(), data.len());
    if unsafe { platform_open() } != 0 { unsafe { platform_close() }; return Err("platform initialization failed".into()); }
    let mut index = 0;
    let mut waiting_idr = false;
    let start = Instant::now();
    let mut deadline = start;
    let mut submitted = 0u64;
    let mut skipped = 0u64;
    let mut failures = 0u64;
    let mut report = start;
    while start.elapsed() < Duration::from_secs(120) && unsafe { platform_tick() } == 0 {
        let now = Instant::now();
        if now < deadline { std::thread::sleep(Duration::from_millis(1)); continue; }
        // Never catch up by bursting frames into the decoder after a stall.
        if now.duration_since(deadline) > Duration::from_millis(100) { waiting_idr = true; }
        let (au,key) = frames[index];
        if !waiting_idr || key {
            let rc = unsafe { platform_feed(au.as_ptr(), au.len(), key as i32) };
            if rc == 0 { submitted+=1; waiting_idr=false; }
            else { failures+=1; waiting_idr=true; eprintln!("M0 feed rc={rc} unit={index}"); }
        } else { skipped+=1; }
        index = (index+1) % frames.len();
        deadline = now + Duration::from_nanos(1_000_000_000 / 60);
        if report.elapsed() >= Duration::from_secs(5) {
            eprintln!("M0 elapsed_ms={} submitted={} skipped={} feed_nonzero={} presented=UNKNOWN",start.elapsed().as_millis(),submitted,skipped,failures);
            report=now;
        }
    }
    unsafe { platform_close() };
    Ok(())
}

/// C entry point. No unwinding crosses the ABI; release builds abort on panic.
#[no_mangle]
pub extern "C" fn m0_run() -> i32 {
    match std::panic::catch_unwind(run) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => { eprintln!("M0 error: {e}"); 1 },
        Err(_) => { eprintln!("M0 panic"); 2 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn aud_preserves_sps_and_keyframe() {
        let b = [0,0,0,1,9,0xf0,0,0,1,7,1,0,0,1,5,2,0,0,1,9,0xf0,0,0,1,1,3];
        let a=access_units(&b).unwrap();
        assert_eq!(a.len(),2); assert!(a[0].1); assert!(!a[1].1);
        assert_eq!([a[0].0,a[1].0].concat(),b);
    }
    #[test] fn rejects_undelimited_and_non_idr() {
        assert!(access_units(&[]).is_err());
        assert!(access_units(&[0,0,1,5,1]).is_err());
        assert!(access_units(&[0,0,1,9,1,0,0,1,1,2]).is_err());
    }
    #[test] fn checked_fixture() {
        let data=include_bytes!("../../../packaging/assets/test.h264");
        let a=access_units(data).unwrap();
        assert_eq!(a.len(),120); assert!(a[0].1); assert!(a[60].1);
    }
}
