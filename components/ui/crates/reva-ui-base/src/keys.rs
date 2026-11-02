//! IUP's key codes and their names (`K_F6`, `K_sF6`, `K_cS`, ...).
//!
//! A key is its base code (a character, or one of the [`NAMED`] keys) with
//! modifier bits, as IUP 3 defines them: the callback for a key is the one
//! named `K_` + modifiers + base name, `K_ANY` receives the code.

/// Modifier bits (IUP's `iup_XkeyShift`, `iup_XkeyCtrl`, ...).
pub const SHIFT: u32 = 0x1000_0000;
pub const CTRL: u32 = 0x2000_0000;
pub const ALT: u32 = 0x4000_0000;
pub const SYS: u32 = 0x8000_0000;

/// Keys the model handles itself (IUP's `K_BS`, `K_TAB`, ...).
pub const BS: u32 = 8;
pub const TAB: u32 = 9;
pub const CR: u32 = 13;
pub const SP: u32 = 32;
pub const HOME: u32 = 0xFF50;
pub const LEFT: u32 = 0xFF51;
pub const UP: u32 = 0xFF52;
pub const RIGHT: u32 = 0xFF53;
pub const DOWN: u32 = 0xFF54;
pub const END: u32 = 0xFF57;
pub const DEL: u32 = 0xFFFF;

/// The modifiers' letters in names, in this order (`K_sF6`, `K_cS`).
const MODS: &[(u32, char)] = &[(SHIFT, 's'), (CTRL, 'c'), (ALT, 'm'), (SYS, 'y')];

/// The keys with a name rather than a character.
pub const NAMED: &[(&str, u32)] = &[
    ("BS", BS),
    ("TAB", TAB),
    ("LF", 10),
    ("CR", CR),
    ("ESC", 27),
    ("SP", SP),
    ("PAUSE", 0xFF13),
    ("HOME", HOME),
    ("LEFT", LEFT),
    ("UP", UP),
    ("RIGHT", RIGHT),
    ("DOWN", DOWN),
    ("PGUP", 0xFF55),
    ("PGDN", 0xFF56),
    ("END", END),
    ("INS", 0xFF63),
    ("F1", 0xFFBE),
    ("F2", 0xFFBF),
    ("F3", 0xFFC0),
    ("F4", 0xFFC1),
    ("F5", 0xFFC2),
    ("F6", 0xFFC3),
    ("F7", 0xFFC4),
    ("F8", 0xFFC5),
    ("F9", 0xFFC6),
    ("F10", 0xFFC7),
    ("F11", 0xFFC8),
    ("F12", 0xFFC9),
    ("DEL", DEL),
];

/// The code of a key: `base` (a character or a [`NAMED`] code) with the
/// modifiers held.  A letter with Ctrl, Alt or Sys is upper case, and
/// Shift is part of a character rather than a modifier (`K_cS`, `K_A`).
pub fn code(base: u32, shift: bool, ctrl: bool, alt: bool) -> u32 {
    let named = NAMED.iter().any(|n| n.1 == base && base != 32);
    let base = match char::from_u32(base) {
        Some(c) if (ctrl || alt) && c.is_ascii_lowercase() => c.to_ascii_uppercase() as u32,
        _ => base,
    };
    let shift = shift && named;
    base | if shift { SHIFT } else { 0 } | if ctrl { CTRL } else { 0 } | if alt { ALT } else { 0 }
}

/// The name of key `code` (`K_sF6`), if it has one: named keys and
/// ASCII letters and digits.
pub fn name(code: u32) -> Option<String> {
    let base = code & 0x0FFF_FFFF;
    let base = match NAMED.iter().find(|n| n.1 == base) {
        Some(n) => n.0.to_string(),
        None => char::from_u32(base)
            .filter(char::is_ascii_alphanumeric)?
            .to_string(),
    };
    let mods: String = MODS
        .iter()
        .filter(|m| code & m.0 != 0)
        .map(|m| m.1)
        .collect();
    Some(format!("K_{mods}{base}"))
}

/// The code of the key called `name` (`K_F6`, `K_cS`), the inverse of
/// [`name`].
pub fn from_name(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("K_")?;
    for split in (0..=rest.len().min(4)).filter(|s| rest.is_char_boundary(*s)) {
        let (mods, base) = rest.split_at(split);
        let Some(bits) = mods
            .chars()
            .try_fold(0, |b, c| MODS.iter().find(|m| m.1 == c).map(|m| b | m.0))
        else {
            break;
        };
        let base = match NAMED.iter().find(|n| n.0 == base) {
            Some(n) => Some(n.1),
            None => Some(base)
                .filter(|b| b.len() == 1 && b.chars().all(|c| c.is_ascii_alphanumeric()))
                .map(|b| b.as_bytes()[0] as u32),
        };
        if let Some(base) = base {
            return Some(base | bits);
        }
    }
    None
}

/// Whether key `code` types its character (no Ctrl, Alt or Sys, not a
/// named key except space).
pub fn printable(code: u32) -> Option<char> {
    if code & (CTRL | ALT | SYS) != 0 {
        return None;
    }
    char::from_u32(code & !SHIFT).filter(|c| *c >= ' ' && (*c as u32) < 0xFF00)
}

/// Pasted `text` as event lines, one key per character: a line break
/// (`\n`, `\r` as terminals send it, or `\r\n`) as one Enter, a tab as a
/// space (Tab moves the focus), other control characters left out.
pub fn pasted(text: &str) -> Vec<String> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let key = |c: char| match c {
        '\n' => Some(CR),
        '\t' => Some(SP),
        c if c.is_control() => None,
        c => Some(c as u32),
    };
    text.chars()
        .filter_map(key)
        .map(|k| format!("key {k}"))
        .collect()
}
