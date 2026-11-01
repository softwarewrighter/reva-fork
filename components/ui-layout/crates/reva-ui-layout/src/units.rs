//! Sizes from attributes, in cells.
//!
//! IUP measures SIZE, CMARGIN and CGAP in character units (a quarter of a
//! character's width, an eighth of its height) and RASTERSIZE, MARGIN and
//! GAP in pixels; a cell is one character, taken as 8 by 16 pixels.

use reva_ui_base::change::Handle;
use reva_ui_tree::tree::Tree;

use crate::geom::{Expand, Size};

/// The two numbers of `"WxH"` (either may be missing: `"W"`, `"xH"`).
pub fn pair(s: &str) -> (Option<u32>, Option<u32>) {
    let (w, h) = s.split_once(['x', 'X']).unwrap_or((s, ""));
    (w.trim().parse().ok(), h.trim().parse().ok())
}

/// `(w, h)` in units of `per` (`(cols, rows)` units per cell) as cells,
/// rounded up or to the nearest.
fn cells(
    (w, h): (Option<u32>, Option<u32>),
    per: (u32, u32),
    up: bool,
) -> (Option<u16>, Option<u16>) {
    let div = |v: u32, d: u32| {
        (if up { v.div_ceil(d) } else { (v + d / 2) / d }).min(u16::MAX as u32) as u16
    };
    (w.map(|w| div(w, per.0)), h.map(|h| div(h, per.1)))
}

/// The size the program asked for, per direction: SIZE, else RASTERSIZE.
pub fn user_size(tree: &Tree, h: Handle) -> (Option<u16>, Option<u16>) {
    let attr = |name| tree.node(h)?.attrs.get(name).map(|v| pair(v));
    match (attr("SIZE"), attr("RASTERSIZE")) {
        (Some(s), _) => cells(s, (4, 8), true),
        (None, Some(r)) => cells(r, (8, 16), true),
        (None, None) => (None, None),
    }
}

/// A box's margins (cells on each side) and gap (cells between children,
/// along `vertical` or not).
pub fn spacing(tree: &Tree, h: Handle, vertical: bool) -> (Size, u16) {
    let get = |name| tree.node(h)?.attrs.get(name).map(|v| pair(v));
    let (mc, mr) = match get("CMARGIN") {
        Some(m) => cells(m, (4, 8), false),
        None => cells(get("MARGIN").unwrap_or_default(), (8, 16), false),
    };
    let (gc, gr) = match get("CGAP") {
        Some((g, _)) => cells((g, g), (4, 8), false),
        None => cells(
            get("GAP").map(|(g, _)| (g, g)).unwrap_or_default(),
            (8, 16),
            false,
        ),
    };
    let gap = if vertical { gr } else { gc };
    (
        Size {
            cols: mc.unwrap_or(0),
            rows: mr.unwrap_or(0),
        },
        gap.unwrap_or(0),
    )
}

/// The directions EXPAND names (YES, HORIZONTAL, VERTICAL, NO).
pub fn expand_attr(tree: &Tree, h: Handle) -> Expand {
    let v = tree.attr(h, "EXPAND").unwrap_or_default();
    let (horizontal, vertical) = match v.as_str() {
        "YES" => (true, true),
        "HORIZONTAL" | "HORIZONTALFREE" => (true, false),
        "VERTICAL" | "VERTICALFREE" => (false, true),
        _ => (false, false),
    };
    Expand {
        horizontal,
        vertical,
    }
}

/// A dialog's SIZE in IUP's fractions of the screen (`FULL`, `HALF`,
/// `THIRD`, `QUARTER`, per direction: `FULLxHALF`), as cells of `screen`.
pub fn screen_size(tree: &Tree, h: Handle, screen: Size) -> (Option<u16>, Option<u16>) {
    let Some(size) = tree.node(h).and_then(|n| n.attrs.get("SIZE")) else {
        return (None, None);
    };
    let (w, hh) = size.split_once(['x', 'X']).unwrap_or((size, ""));
    let part = |v: &str, all: u16| match v.trim() {
        "FULL" => Some(all),
        "HALF" => Some(all / 2),
        "THIRD" => Some(all / 3),
        "QUARTER" => Some(all / 4),
        _ => None,
    };
    (part(w, screen.cols), part(hh, screen.rows))
}
