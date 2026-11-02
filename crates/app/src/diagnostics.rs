//! One minute of bounded, secret-free UI liveness diagnostics.
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};
static NATIVE_PHASE: AtomicU32 = AtomicU32::new(0);
#[no_mangle]
pub extern "C" fn rp_trace_native(phase: u32) {
    NATIVE_PHASE.store(phase, Ordering::Relaxed);
}
pub struct Activity {
    pub phase: AtomicU32,
    pub polls: AtomicU32,
    pub events: AtomicU32,
    pub navigation: AtomicU32,
    pub frames: AtomicU32,
    stop: AtomicBool,
}
pub struct Monitor(pub Arc<Activity>);
impl Monitor {
    pub fn start() -> Self {
        let state = Arc::new(Activity {
            phase: AtomicU32::new(0),
            polls: AtomicU32::new(0),
            events: AtomicU32::new(0),
            navigation: AtomicU32::new(0),
            frames: AtomicU32::new(0),
            stop: AtomicBool::new(false),
        });
        let worker = state.clone();
        std::thread::spawn(move || {
            for second in 1..=60 {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if worker.stop.load(Ordering::Relaxed) {
                    break;
                }
                if second % 5 == 0 {
                    let phase = match worker.phase.load(Ordering::Relaxed) {
                        1 => "native_poll",
                        2 => "handle_event",
                        3 => "update",
                        4 => "draw_present",
                        _ => "idle",
                    };
                    let native = match NATIVE_PHASE.load(Ordering::Relaxed) {
                        1 => "controller_scan",
                        2 => "SDL_PollEvent",
                        3 => "controller_event",
                        4 => "SDL_RenderClear",
                        5 => "SDL_RenderFillRect",
                        6 => "SDLTest_DrawString",
                        7 => "SDL_RenderPresent",
                        8 => "SDL_text_input",
                        _ => "returned",
                    };
                    eprintln!(
                        "UI heartbeat t={} phase={} native={} polls={} events={} navigation={} frames={}",
                        second,
                        phase,
                        native,
                        worker.polls.load(Ordering::Relaxed),
                        worker.events.load(Ordering::Relaxed),
                        worker.navigation.load(Ordering::Relaxed),
                        worker.frames.load(Ordering::Relaxed)
                    );
                }
            }
        });
        Self(state)
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.0.stop.store(true, Ordering::Relaxed);
    }
}
