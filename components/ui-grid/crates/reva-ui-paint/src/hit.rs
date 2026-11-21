//! Where a text element's characters are: the cells its text is drawn in,
//! how far it is scrolled, and which line and column a cell shows, so a
//! click puts the caret on the character under it.

use reva_ui_base::change::Handle;
use reva_ui_base::kind::Kind;
use reva_ui_layout::geom::Rect;
use reva_ui_mirror::mirror::Mirror;

use crate::fields::parse_pos;

/// The cells an element of `kind` placed at `r` draws its text or items
/// in: a one-line text between its brackets, a multiline or a list inside
/// its box.
pub fn content_area(kind: Kind, r: Rect) -> Option<Rect> {
    match kind {
        Kind::Text => Some(Rect {
            col: r.col + 1,
            cols: r.cols.saturating_sub(2),
            rows: 1,
            ..r
        }),
        Kind::MultiLine | Kind::List => Some(Rect {
            col: r.col + 1,
            row: r.row + 1,
            cols: r.cols.saturating_sub(2),
            rows: r.rows.saturating_sub(2),
        }),
        _ => None,
    }
}

/// The first line and column shown in `area` with the caret at `caret`
/// (line, column from 0), the view having been at `was`: it stays while
/// the caret is in it, else moves just far enough to show the caret.
pub fn scroll(was: (usize, usize), caret: (usize, usize), area: Rect) -> (usize, usize) {
    let follow = |was: usize, at: usize, len: u16| {
        was.min(at).max(at.saturating_sub(len.max(1) as usize - 1))
    };
    (
        follow(was.0, caret.0, area.rows),
        follow(was.1, caret.1, area.cols),
    )
}

/// The line and column, from 1, that text element `h` shows at `col`,
/// `row` of `area`, as it was last drawn (a cell outside `area` counts as
/// the nearest inside); the model puts a column past the end of the line
/// at its end.
pub fn caret_at(m: &Mirror, h: Handle, area: Rect, col: u16, row: u16) -> (usize, usize) {
    let caret = m
        .tree
        .attr(h, "CARET")
        .and_then(|c| parse_pos(&c))
        .unwrap_or((0, 0));
    let was = m.scroll.get(&h).copied().unwrap_or_default();
    let (top, left) = scroll(was, caret, area);
    let at =
        |v: u16, start: u16, len: u16| (v.clamp(start, start + len.max(1) - 1) - start) as usize;
    (
        top + at(row, area.row, area.rows) + 1,
        left + at(col, area.col, area.cols) + 1,
    )
}
