// SPDX-License-Identifier: AGPL-3.0-only
//! Direct native PS5 Remote Play client. Sony account services and payload management are absent.
mod config;
mod diagnostics;
mod ffi;
mod haptics;
mod hdr;
mod queues;
mod session;
mod ui;
use std::{
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
fn run() -> Result<(), &'static str> {
    std::panic::set_hook(Box::new(|_| {
        eprintln!("Native client panic (details suppressed)")
    }));
    let dir = std::env::var_os("PS5REMOTE_DATA_DIR").ok_or("Private storage path unavailable")?;
    eprintln!("Startup reading private configuration");
    let store = config::Store::open(PathBuf::from(dir))?;
    let prefs = store.preferences()?;
    let paired = store.pairing()?;
    eprintln!("Startup configuration ready; opening native platform");
    if unsafe { ffi::rp_open() } != 0 {
        unsafe {
            ffi::rp_close();
        }
        return Err("SDL/NDL initialization failed");
    }
    let mut app = ui::App::new(store, prefs, paired);
    let monitor = diagnostics::Monitor::start();
    let mut last_log = Instant::now();
    let mut last_draw = Instant::now() - Duration::from_millis(20);
    loop {
        let mut keep = true;
        for _ in 0..512 {
            let mut event = ffi::Event::default();
            monitor.0.phase.store(1, Ordering::Relaxed);
            let available = unsafe { ffi::rp_poll(&mut event) };
            monitor.0.polls.fetch_add(1, Ordering::Relaxed);
            if available == 0 {
                break;
            }
            monitor.0.events.fetch_add(1, Ordering::Relaxed);
            if (1..=6).contains(&event.kind) || event.kind == 11 {
                monitor.0.navigation.fetch_add(1, Ordering::Relaxed);
            }
            monitor.0.phase.store(2, Ordering::Relaxed);
            if !app.event(&event) {
                keep = false;
                break;
            }
        }
        if !keep {
            break;
        }
        monitor.0.phase.store(3, Ordering::Relaxed);
        app.update();
        if last_draw.elapsed() >= Duration::from_nanos(1_000_000_000 / 60) {
            monitor.0.phase.store(4, Ordering::Relaxed);
            app.draw();
            monitor.0.frames.fetch_add(1, Ordering::Relaxed);
            last_draw = Instant::now();
        }
        if last_log.elapsed() > Duration::from_secs(5) {
            app.log_stats();
            last_log = Instant::now();
        }
        monitor.0.phase.store(0, Ordering::Relaxed);
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
