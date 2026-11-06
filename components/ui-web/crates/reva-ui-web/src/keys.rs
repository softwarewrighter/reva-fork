//! Browser keys (`KeyboardEvent.key`) as IUP key codes, and the element's
//! key and paste handlers.

use reva_ui_base::keys::{code, from_name};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::state::send;

/// The browser's names of keys IUP names otherwise.
const NAMES: &[(&str, &str)] = &[
    ("Enter", "CR"),
    ("Tab", "TAB"),
    ("Backspace", "BS"),
    ("Delete", "DEL"),
    ("Escape", "ESC"),
    ("Home", "HOME"),
    ("End", "END"),
    ("ArrowLeft", "LEFT"),
    ("ArrowRight", "RIGHT"),
    ("ArrowUp", "UP"),
    ("ArrowDown", "DOWN"),
    ("PageUp", "PGUP"),
    ("PageDown", "PGDN"),
    ("Insert", "INS"),
    ("Pause", "PAUSE"),
];

/// The IUP code of the key the browser calls `key` with these modifiers;
/// `None` for keys IUP has no code for (Shift alone, media keys, ...).
pub fn key_code(key: &str, shift: bool, ctrl: bool, alt: bool) -> Option<u32> {
    let iup = NAMES.iter().find(|n| n.0 == key).map_or(key, |n| n.1);
    let base = match from_name(&format!("K_{iup}")).filter(|_| iup.chars().count() > 1) {
        Some(b) => b,
        None => {
            let mut chars = key.chars();
            let c = chars.next().filter(|_| chars.next().is_none())?;
            c as u32
        }
    };
    Some(code(base, shift, ctrl, alt))
}

/// Whether the browser keeps `key` with Ctrl (`ctrl`) or Cmd (`meta`):
/// copy and paste, and cut with Cmd only, so that a paste event follows;
/// Ctrl-X goes to the program (Ctrl-X Ctrl-E evaluates in the notebook).
pub fn clipboard_key(key: &str, ctrl: bool, meta: bool) -> bool {
    match key {
        "c" | "C" | "v" | "V" => ctrl || meta,
        "x" | "X" => meta,
        _ => false,
    }
}

/// Pasted text as event lines: reva-ui-base's, shared with the terminal.
pub use reva_ui_base::keys::pasted;

/// A key: its IUP code to the VM, the browser's own use of it prevented.
pub(crate) fn on_key(e: KeyboardEvent) {
    if clipboard_key(&e.key(), e.ctrl_key(), e.meta_key()) {
        return;
    }
    if let Some(code) = key_code(
        &e.key(),
        e.shift_key(),
        e.ctrl_key() || e.meta_key(),
        e.alt_key(),
    ) {
        e.prevent_default();
        send(vec![format!("key {code}")]);
    }
}

/// A paste: the clipboard's text typed into the focused element.
pub(crate) fn on_paste(e: Event) {
    let Some(text) = e
        .dyn_ref::<web_sys::ClipboardEvent>()
        .and_then(|c| c.clipboard_data())
        .and_then(|d| d.get_data("text/plain").ok())
    else {
        return;
    };
    e.prevent_default();
    send(pasted(&text));
}
