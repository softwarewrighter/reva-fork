//! What the drawing functions add to a canvas's display list, from their
//! arguments and the canvas's attributes.

use crate::calls::CdFn;
use crate::draw::Op;
use crate::state::{Bitmap, Canvas};

/// The interior style that draws no fill (CD_HOLLOW), and CD_FILL.
const HOLLOW: i32 = 4;
const FILL: i32 = 0;

/// A drawing on canvas `c` (0 returned): `n(i)` is argument `i`, `text(i)`
/// the string it points to, `bitmap` the one the first names.
pub(crate) fn draw(
    c: &mut Canvas,
    f: CdFn,
    n: &dyn Fn(usize) -> i32,
    text: impl Fn(usize) -> String,
    bitmap: Option<Bitmap>,
) -> u32 {
    let op = match f {
        CdFn::Clear => {
            c.ops.clear();
            Some(Op::Clear(c.background))
        }
        CdFn::Begin => {
            c.poly = Some((n(0), Vec::new()));
            None
        }
        CdFn::Vertex => {
            if let Some((_, pts)) = c.poly.as_mut() {
                pts.push((n(0), n(1)));
            }
            None
        }
        CdFn::End => c.poly.take().map(|(mode, points)| Op::Poly {
            points,
            color: c.foreground,
            filled: mode == FILL,
        }),
        _ => shape(c, f, n).or_else(|| label(c, f, n, text, bitmap)),
    };
    if let Some(op) = op {
        c.ops.push(op);
        c.dirty = true;
    }
    0
}

/// Lines, boxes, marks and pixels.
fn shape(c: &Canvas, f: CdFn, n: &dyn Fn(usize) -> i32) -> Option<Op> {
    let color = c.foreground;
    Some(match f {
        CdFn::Line => Op::Line {
            from: (n(0), n(1)),
            to: (n(2), n(3)),
            color,
            style: c.line_style,
            width: c.line_width,
        },
        CdFn::Box | CdFn::Rect => Op::Box {
            lo: (n(0), n(2)),
            hi: (n(1), n(3)),
            color,
            filled: f == CdFn::Box && c.interior != HOLLOW,
        },
        CdFn::Mark => Op::Mark {
            at: (n(0), n(1)),
            kind: c.mark_type,
            size: c.mark_size,
            color,
        },
        CdFn::Pixel => Op::Box {
            lo: (n(0), n(1)),
            hi: (n(0), n(1)),
            color: rgb(n(2)),
            filled: true,
        },
        _ => return None,
    })
}

/// Texts and bitmaps.
fn label(
    c: &Canvas,
    f: CdFn,
    n: &dyn Fn(usize) -> i32,
    text: impl Fn(usize) -> String,
    bitmap: Option<Bitmap>,
) -> Option<Op> {
    let (color, size) = (c.foreground, c.text_size);
    match f {
        CdFn::Text | CdFn::VectorText => Some(Op::Text {
            at: (n(0), n(1)),
            color,
            size,
            text: text(2),
        }),
        CdFn::CanvasVectorText => Some(Op::Text {
            at: (n(1), n(2)),
            color,
            size,
            text: text(3),
        }),
        CdFn::PutBitmap => bitmap.map(|b| Op::Image {
            at: (n(1), n(2)),
            size: (n(3), n(4)),
            w: b.w,
            h: b.h,
            pixels: b.pixels,
        }),
        _ => None,
    }
}

/// A CD colour's red, green and blue (its top byte is alpha, which the
/// display list does not keep).
pub(crate) fn rgb(v: i32) -> u32 {
    v as u32 & 0xff_ffff
}
