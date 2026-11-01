//! What the page draws: the turtle's drawing ops, and the changes to the
//! TUI of IUP programs (docs/ui-model.md).

use reva_ui_base::codec::encode;
use reva_vm::prelude::*;
use reva_web_state::state::{CANVAS, FSBUF, with};

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

/// Take the TUI's changes not yet sent (by a `ui` host call or an earlier
/// take) into the buffer at `reva_fs_data`, one line each (reva-ui-base's
/// codec); returns their length (0: nothing new).  The worker calls it
/// after every slice, so the page also sees a dialog shown or hidden
/// outside the event loop.
#[unsafe(no_mangle)]
pub extern "C" fn reva_ui_take() -> u32 {
    let lines: Vec<String> = with(|w| w.vm.ui.tree.take_changes(), Vec::new())
        .iter()
        .map(encode)
        .collect();
    let text = lines.join("\n");
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        *b = text.into_bytes();
        b.len() as u32
    })
}
