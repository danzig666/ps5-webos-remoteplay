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
pub extern "C" fn rp_feedback(_: u8, _: u8, _: *const u8, _: *const u8, _: i32, _: i32) {}
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
