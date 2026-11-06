//! What a canvas shows: a list of operations in CD's coordinates (pixels,
//! the origin at the bottom left, y up), each with the colour and style it
//! was drawn with, and their text form, the value of the IUP canvas's
//! `DRAW` attribute: items separated by `;`, fields by spaces, colours as
//! six hex digits, a text last (with `\;` and `\\` for `;` and `\`).

use crate::codec::{item, parse};

/// A colour, `0xRRGGBB` (CD's `long` colours without alpha).
pub type Rgb = u32;

/// One drawing operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// The whole canvas in a colour (`cdClear`, the background).
    Clear(Rgb),
    /// `cdLine`: a line in a style (CD's line styles) and width.
    Line {
        from: (i32, i32),
        to: (i32, i32),
        color: Rgb,
        style: i32,
        width: i32,
    },
    /// `cdBox` (filled) or `cdRect` (outlined): corners `lo` and `hi`.
    Box {
        lo: (i32, i32),
        hi: (i32, i32),
        color: Rgb,
        filled: bool,
    },
    /// `cdMark`: a marker of CD's mark type and size centred at `at`.
    Mark {
        at: (i32, i32),
        kind: i32,
        size: i32,
        color: Rgb,
    },
    /// `cdText` / `cdVectorText`: a text with its left baseline at `at`.
    Text {
        at: (i32, i32),
        color: Rgb,
        size: i32,
        text: String,
    },
    /// `cdPutBitmap`: `w`×`h` pixels (row 0 at the bottom, as in CD)
    /// drawn at `at`, scaled to `size`.
    Image {
        at: (i32, i32),
        size: (i32, i32),
        w: i32,
        h: i32,
        pixels: Vec<Rgb>,
    },
    /// `cdBegin`…`cdEnd`: a polygon, filled or a closed line.
    Poly {
        points: Vec<(i32, i32)>,
        color: Rgb,
        filled: bool,
    },
}

/// `ops` in text form.
pub fn encode(ops: &[Op]) -> String {
    ops.iter().map(item).collect::<Vec<_>>().join(";")
}

/// The operations of a `DRAW` value; items it cannot read are skipped.
pub fn decode(s: &str) -> Vec<Op> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => cur.extend(chars.next()),
            ';' => items.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    items.push(cur);
    items.iter().filter_map(|i| parse(i)).collect()
}
