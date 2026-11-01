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
