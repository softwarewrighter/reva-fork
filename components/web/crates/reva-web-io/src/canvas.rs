//! The turtle's drawing ops for the page.

use reva_vm::prelude::*;
use reva_web_state::state::{CANVAS, with};

/// Take the drawing ops not yet taken into the canvas buffer; returns how
/// many `f32`s it holds (0: nothing new).
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_take() -> u32 {
    let ops = with(|w| w.vm.canvas().take_ops(), Vec::new());
    let n = ops.len() as u32;
    CANVAS.with(|c| *c.borrow_mut() = ops);
    n
}

/// The canvas buffer's address (after [`reva_canvas_take`]).
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_ptr() -> *const f32 {
    CANVAS.with(|c| c.borrow().as_ptr())
}

/// Hand out the whole drawing again from the next [`reva_canvas_take`].
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_resend() {
    with(|w| w.vm.canvas().resend(), ())
}

/// 1 once after `restart` replaced the machine (and its drawing), else 0.
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_reset() -> i32 {
    with(|w| std::mem::take(&mut w.canvas_reset) as i32, 0)
}
