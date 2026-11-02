// SPDX-License-Identifier: AGPL-3.0-only
//! Direct native PS5 Remote Play client. Sony account services and payload management are absent.
mod config;
mod ffi;
mod haptics;
mod hdr;
mod queues;
mod session;
mod ui;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn run() -> Result<(), &'static str> {
    std::panic::set_hook(Box::new(|_| {
        eprintln!("Native client panic (details suppressed)")
    }));
    let dir = std::env::var_os("PS5REMOTE_DATA_DIR").ok_or("Private storage path unavailable")?;
    let store = config::Store::open(PathBuf::from(dir))?;
    let prefs = store.preferences()?;
    let paired = store.pairing()?;
    if unsafe { ffi::rp_open() } != 0 {
        unsafe {
            ffi::rp_close();
        }
        return Err("SDL/NDL initialization failed");
    }
    let mut app = ui::App::new(store, prefs, paired);
    let mut last_log = Instant::now();
    let mut last_draw = Instant::now() - Duration::from_millis(20);
    loop {
        let mut keep = true;
        for _ in 0..512 {
            let mut event = ffi::Event::default();
            if unsafe { ffi::rp_poll(&mut event) } == 0 {
                break;
            }
            if !app.event(&event) {
                keep = false;
                break;
            }
        }
        if !keep {
            break;
        }
        app.update();
        if last_draw.elapsed() >= Duration::from_nanos(1_000_000_000 / 60) {
            app.draw();
            last_draw = Instant::now();
        }
        if last_log.elapsed() > Duration::from_secs(5) {
            app.log_stats();
            last_log = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    app.shutdown();
    unsafe {
        ffi::rp_close();
    }
    Ok(())
}
/// Thin C bootstrap calls this; no Rust unwind may cross the C ABI.
#[no_mangle]
pub extern "C" fn rp_run() -> i32 {
    match std::panic::catch_unwind(run) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            eprintln!("Client error: {e}");
            1
        }
        Err(_) => {
            unsafe {
                ffi::rp_close();
            }
            2
        }
    }
}

#[cfg(test)]
mod native_test;
