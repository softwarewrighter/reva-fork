//! Boxes: a vbox's or an hbox's natural size, and its children's places.
//!
//! Worked out for a vbox; an hbox's sizes and rectangles are turned on
//! their side, so `rows` runs along the box and `cols` across it.

use reva_ui_base::change::Handle;
use reva_ui_tree::tree::Tree;

use crate::geom::{Placed, Rect, Size};
use crate::natural::{expands, natural};
use crate::place::{inset, place};
use crate::units::spacing;

/// A count of cells, at most what a [`Size`] holds.
pub fn clamp(n: u32) -> u16 {
    n.min(u32::from(u16::MAX)) as u16
}

/// `r` turned on its side unless `vertical`.
fn turned(r: Rect, vertical: bool) -> Rect {
    if vertical {
        r
    } else {
        Rect {
            col: r.row,
            row: r.col,
            cols: r.rows,
            rows: r.cols,
        }
    }
}

/// A box's natural size: its children's sizes `kids` one after another,
/// with the gaps between them and the margins around.
pub fn box_size(tree: &Tree, h: Handle, kids: &[Size], vertical: bool) -> Size {
    let turn = |s: Size| {
        turned(
            Rect {
                cols: s.cols,
                rows: s.rows,
                ..Rect::default()
            },
            vertical,
        )
    };
    let (margin, gap) = spacing(tree, h, vertical);
    let margin = turn(margin);
    let gaps = u32::from(gap) * kids.len().saturating_sub(1) as u32;
    let rows = kids.iter().map(|s| u32::from(turn(*s).rows)).sum::<u32>() + gaps;
    let cols = kids.iter().map(|s| turn(*s).cols).max().unwrap_or(0);
    let size = turned(
        Rect {
            cols: clamp(u32::from(cols) + 2 * u32::from(margin.cols)),
            rows: clamp(rows + 2 * u32::from(margin.rows)),
            ..Rect::default()
        },
        vertical,
    );
    Size {
        cols: size.cols,
        rows: size.rows,
    }
}

/// Lay out the children `kids` of box `h` in `rect`: in order at their
/// natural sizes, the space left shared among those that expand along the
/// box; across it, the whole width for those that expand, else where
/// ALIGNMENT puts them.
pub fn boxed(
    tree: &Tree,
    h: Handle,
    kids: &[Handle],
    rect: Rect,
    vertical: bool,
    out: &mut Vec<Placed>,
) {
    let (margin, gap) = spacing(tree, h, vertical);
    let a = turned(inset(rect, margin), vertical);
    let sizes = measure(tree, kids, vertical);
    let used = sizes.iter().map(|s| u32::from(s.0.rows)).sum::<u32>()
        + u32::from(gap) * kids.len().saturating_sub(1) as u32;
    let mut shares = shares(
        u32::from(a.rows).saturating_sub(used),
        sizes.iter().filter(|s| s.1).count() as u32,
    );
    let how = tree.attr(h, "ALIGNMENT").unwrap_or_default();
    let mut at = a.row;
    for (c, (n, along, across)) in kids.iter().zip(sizes) {
        let rows = n
            .rows
            .saturating_add(if along { shares.next().unwrap_or(0) } else { 0 });
        let cols = if across { a.cols } else { n.cols.min(a.cols) };
        let col = a.col.saturating_add(align(&how, a.cols - cols));
        place(
            tree,
            *c,
            turned(
                Rect {
                    col,
                    row: at,
                    cols,
                    rows,
                },
                vertical,
            ),
            out,
        );
        at = at.saturating_add(rows).saturating_add(gap);
    }
}

/// The natural size of each of `kids`, turned, and whether it expands
/// along and across the box.
fn measure(tree: &Tree, kids: &[Handle], vertical: bool) -> Vec<(Rect, bool, bool)> {
    let one = |c: Handle| {
        let (n, e) = (natural(tree, c), expands(tree, c));
        let r = turned(
            Rect {
                cols: n.cols,
                rows: n.rows,
                ..Rect::default()
            },
            vertical,
        );
        if vertical {
            (r, e.vertical, e.horizontal)
        } else {
            (r, e.horizontal, e.vertical)
        }
    };
    kids.iter().map(|c| one(*c)).collect()
}

/// `extra` cells shared among `n` growing children, the first ones
/// getting one more when it does not divide.
fn shares(extra: u32, n: u32) -> impl Iterator<Item = u16> {
    (0..n).map(move |i| clamp(extra / n + u32::from(i < extra % n)))
}

/// Where a child `room` cells smaller than its box goes across it.
fn align(how: &str, room: u16) -> u16 {
    match how.split(':').next().unwrap_or("") {
        "ACENTER" => room / 2,
        "ARIGHT" | "ABOTTOM" => room,
        _ => 0,
    }
}
