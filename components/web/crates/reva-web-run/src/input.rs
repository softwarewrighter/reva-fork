//! Keyboard input and start input.

use reva_vm::prelude::*;
use reva_web_state::state::*;

/// A buffer of `len` bytes for the caller to fill before [`reva_feed`].
#[unsafe(no_mangle)]
pub extern "C" fn reva_input_buffer(len: u32) -> *mut u8 {
    with(
        |w| {
            w.input.resize(len as usize, 0);
            w.input.as_mut_ptr()
        },
        std::ptr::null_mut(),
    )
}

/// Queue the first `len` bytes of the input buffer as keyboard input.
#[unsafe(no_mangle)]
pub extern "C" fn reva_feed(len: u32) {
    with(
        |w| {
            let n = (len as usize).min(w.input.len());
            w.vm.feed(&w.input[..n]);
        },
        (),
    )
}

/// Like [`reva_feed`], and keep the bytes to feed again after `restart`.
#[unsafe(no_mangle)]
pub extern "C" fn reva_feed_start(len: u32) {
    with(
        |w| {
            let n = (len as usize).min(w.input.len());
            w.vm.feed(&w.input[..n]);
            w.replay.extend_from_slice(&w.input[..n]);
        },
        (),
    )
}

/// Signal end of input (the interpreter exits once it drains).
#[unsafe(no_mangle)]
pub extern "C" fn reva_close_input() {
    with(|w| w.vm.close_input(), ())
}
