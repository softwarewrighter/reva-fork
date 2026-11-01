//! What the page calls (www/tui.js).

use reva_ui_layout::geom::Size;
use reva_ui_mirror::mirror::Mirror;
use wasm_bindgen::prelude::*;

use crate::state::{redraw, with};
use crate::view::Tui;

/// Draw the TUI in `root`; event lines go to `on_event(line)`.
#[wasm_bindgen]
pub fn mount(root: web_sys::Element, on_event: js_sys::Function) {
    with(|s| s.on_event = Some(on_event));
    yew::Renderer::<Tui>::with_root(root).render();
}

/// Apply the change lines of a `ui` `wait` call; whether a dialog is
/// shown (the page shows the TUI only then).
#[wasm_bindgen]
pub fn changes(lines: &str) -> bool {
    with(|s| s.mirror.apply_lines(lines));
    redraw();
    shown()
}

/// The screen is now `cols` by `rows` cells.
#[wasm_bindgen]
pub fn resize(cols: u16, rows: u16) {
    with(|s| {
        s.size = Size {
            cols: cols.max(1),
            rows: rows.max(1),
        }
    });
    redraw();
}

/// Whether a dialog is shown.
#[wasm_bindgen]
pub fn shown() -> bool {
    with(|s| !s.mirror.shown.is_empty())
}

/// Forget the UI (the VM restarted).
#[wasm_bindgen]
pub fn reset() {
    with(|s| s.mirror = Mirror::default());
    redraw();
}

/// Type `text` into the focused element (a paste the page caught).
#[wasm_bindgen]
pub fn paste(text: &str) {
    crate::state::send(crate::keys::pasted(text));
}
