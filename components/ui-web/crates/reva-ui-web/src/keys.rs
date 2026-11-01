//! Browser keys (`KeyboardEvent.key`) as IUP key codes.

use reva_ui_base::keys::{code, from_name};

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

/// Whether the browser keeps `key` with Ctrl or Cmd (`ctrl`): copy, cut
/// and paste, so that a paste event follows.
pub fn clipboard_key(key: &str, ctrl: bool) -> bool {
    ctrl && matches!(key, "c" | "C" | "v" | "V" | "x" | "X")
}

/// Pasted `text` as event lines, one key per character: newlines as
/// Enter, tabs as spaces (Tab moves the focus), other control characters
/// left out.
pub fn pasted(text: &str) -> Vec<String> {
    let key = |c: char| match c {
        '\n' => Some(13),
        '\t' => Some(32),
        c if c.is_control() => None,
        c => Some(c as u32),
    };
    text.chars()
        .filter_map(key)
        .map(|k| format!("key {k}"))
        .collect()
}
