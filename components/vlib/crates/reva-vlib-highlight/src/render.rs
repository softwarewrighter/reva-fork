//! The library's names, and source rendered with ANSI colour.

use crate::lex::tokens;
use crate::table::Table;
use reva_core::Gate;

/// Handle `(lib)` returns for the virtual highlight library ("HILT").
pub const HL_HANDLE: u32 = 0x4849_4c54;

/// Library names accepted for it.
pub const HL_NAMES: &[&str] = &[
    "highlight",
    "libhighlight",
    "libhighlight.so",
    "highlight.dll",
];

/// The gateway of highlight function `name`.
pub fn lookup(name: &str) -> Option<Gate> {
    match name {
        "hl_type" => Some(Gate::HlType),
        "hl_color" => Some(Gate::HlColor),
        _ => None,
    }
}

/// `src` highlighted with ANSI SGR escapes, line by line.
pub fn render(src: &[u8], table: &Table) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 2);
    for (k, line) in src.split(|&b| b == b'\n').enumerate() {
        if k > 0 {
            out.push(b'\n');
        }
        for (class, a, b) in tokens(line, table) {
            match table.sgr.get(class) {
                Some(sgr) if !class.is_empty() => {
                    out.extend_from_slice(format!("\x1b[{sgr}m").as_bytes());
                    out.extend_from_slice(&line[a..b]);
                    out.extend_from_slice(b"\x1b[0m");
                }
                _ => out.extend_from_slice(&line[a..b]),
            }
        }
    }
    out
}
