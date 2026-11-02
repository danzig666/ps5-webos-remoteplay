// SPDX-License-Identifier: AGPL-3.0-only
//! Headless UI test double. Compiled only into host unit tests, never the native package.
use crate::{
    ffi::{Event, RawInput},
    hdr::HdrMetadata,
};
use std::{
    cell::RefCell,
    ffi::{c_char, CStr},
};
thread_local! {pub static LINES:RefCell<Vec<String>>=const{RefCell::new(Vec::new())};}
thread_local! {pub static TEXT_INPUT: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };}
type Feedback = (u8, u8, Option<[u8; 3]>, Option<[u8; 22]>);
thread_local! {pub static FEEDBACK: RefCell<Vec<Feedback>> = const { RefCell::new(Vec::new()) };}
#[no_mangle]
pub extern "C" fn rp_open() -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_poll(_: *mut Event) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_begin(_: i32) {
    LINES.with(|s| s.borrow_mut().clear());
}
#[no_mangle]
pub extern "C" fn rp_text(_: i32, s: *const c_char, _: i32) {
    let text = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
    LINES.with(|s| s.borrow_mut().push(text));
}
#[no_mangle]
pub extern "C" fn rp_present() {}
#[no_mangle]
pub extern "C" fn rp_text_input(active: i32) {
    TEXT_INPUT.with(|value| value.set(active));
}
#[no_mangle]
pub extern "C" fn rp_close() {}
#[no_mangle]
pub extern "C" fn rp_media_open(
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: *const HdrMetadata,
) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_video(_: *const u8, _: usize, _: i32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_audio(_: *const u8, _: usize, _: i32) -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_media_close() {}
#[no_mangle]
pub extern "C" fn rp_feedback(l: u8, r: u8, rgb: *const u8, triggers: *const u8, _: i32, _: i32) {
    let led = if rgb.is_null() {
        None
    } else {
        Some(unsafe { *(rgb as *const [u8; 3]) })
    };
    let triggers = if triggers.is_null() {
        None
    } else {
        Some(unsafe { *(triggers as *const [u8; 22]) })
    };
    FEEDBACK.with(|f| f.borrow_mut().push((l, r, led, triggers)));
}
#[no_mangle]
pub extern "C" fn rp_effects_reset() {}
#[no_mangle]
pub extern "C" fn rp_haptics_open(_: *const c_char) -> i32 {
    -1
}
#[no_mangle]
pub extern "C" fn rp_haptics_feed(_: *const i16, _: usize) -> i32 {
    -1
}
#[no_mangle]
pub extern "C" fn rp_haptics_close() {}
#[no_mangle]
pub extern "C" fn rp_audio_device_count() -> i32 {
    0
}
#[no_mangle]
pub extern "C" fn rp_audio_device_name(_: i32, out: *mut c_char, size: usize) {
    if size > 0 {
        unsafe {
            *out = 0;
        }
    }
}
#[no_mangle]
pub extern "C" fn rp_diagnostics(out: *mut c_char, size: usize) -> i32 {
    let text = b"SIMULATED_NATIVE\0";
    if size >= text.len() {
        unsafe {
            std::ptr::copy_nonoverlapping(text.as_ptr(), out as *mut u8, text.len());
        }
    }
    0
}
#[allow(dead_code)]
fn c_layout(_: RawInput) {}
