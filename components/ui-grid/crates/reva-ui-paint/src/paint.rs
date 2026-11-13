//! Whole screens: every shown dialog, the top one last, where it was
//! shown; and what is under a click.

use reva_ui_base::change::Handle;
use reva_ui_base::kind::{Kind, takes_focus};
use reva_ui_layout::geom::{Placed, Rect, Size};
use reva_ui_layout::place::{layout, wanted};
use reva_ui_mirror::mirror::{Mirror, Shown};

use crate::draw::{close_cols, draw};
use reva_ui_grid::grid::{Grid, Role};

/// Where dialog `s`, `size` cells big, goes on a `screen`: at its IUP
/// position in pixels (8 by 16 to a cell), centred when that is negative
/// or one of IUP's named positions; kept on the screen.
fn origin(s: &Shown, size: Size, screen: Size) -> (u16, u16) {
    let at = |v: i32, px: i32, len: u16, room: u16| {
        let fit = room.saturating_sub(len);
        if !(0..0xFFF0).contains(&v) {
            fit / 2
        } else {
            ((v / px).min(i32::from(u16::MAX)) as u16).min(fit)
        }
    };
    (
        at(s.x, 8, size.cols, screen.cols),
        at(s.y, 16, size.rows, screen.rows),
    )
}

/// Every shown dialog's elements, where they go on a `screen`, the
/// bottom dialog first and each dialog before what is in it.
pub fn placements(m: &Mirror, screen: Size) -> Vec<Placed> {
    let mut out = Vec::new();
    for s in &m.shown {
        let placed = layout(&m.tree, s.handle, screen);
        let Some(d) = placed.first() else { continue };
        let (col, row) = origin(
            s,
            Size {
                cols: d.rect.cols,
                rows: d.rect.rows,
            },
            screen,
        );
        out.extend(placed.into_iter().map(|p| Placed {
            rect: Rect {
                col: p.rect.col + col,
                row: p.rect.row + row,
                ..p.rect
            },
            ..p
        }));
    }
    out
}

/// The grid a host whose pane scrolls paints on: `screen`, grown to hold
/// every shown dialog at the size it asks for (a dialog larger than the
/// pane is scrolled to, not cut off).
pub fn room(m: &Mirror, screen: Size) -> Size {
    m.shown
        .iter()
        .map(|s| wanted(&m.tree, s.handle, screen))
        .fold(screen, |a, w| Size {
            cols: a.cols.max(w.cols),
            rows: a.rows.max(w.rows),
        })
}

/// The screen: `m`'s shown dialogs painted on a grid of `screen` cells.
pub fn paint(m: &Mirror, screen: Size) -> Grid {
    let mut g = Grid::new(screen, Role::Screen);
    for p in placements(m, screen) {
        draw(&mut g, m, p.handle, p.rect);
    }
    g
}

/// What a click at `col`, `row` does, as event lines for the VM: a
/// button, toggle or menu item is focused and clicked, another element
/// that takes the focus is focused, the top dialog's close box `×` closes
/// it; nothing on a modal dialog's backdrop.
pub fn click(m: &Mirror, screen: Size, col: u16, row: u16) -> Vec<String> {
    let inside =
        |r: &Rect| (r.col..r.col + r.cols).contains(&col) && (r.row..r.row + r.rows).contains(&row);
    let placed = placements(m, screen);
    let top = m.top().and_then(|d| placed.iter().find(|p| p.handle == d));
    if let Some(d) = top.filter(|d| row == d.rect.row && close_cols(d.rect).contains(&col)) {
        return vec![format!("close {}", d.handle)];
    }
    let hit: Option<Handle> = placed
        .into_iter()
        .filter(|p| inside(&p.rect))
        .map(|p| p.handle)
        .rfind(|h| m.tree.node(*h).is_some_and(|n| takes_focus(n.kind)));
    let Some(h) = hit.filter(|h| {
        !m.shown
            .last()
            .is_some_and(|s| s.modal && !m.tree.contains(s.handle, *h))
    }) else {
        return Vec::new();
    };
    match m.tree.node(h).map(|n| n.kind) {
        Some(Kind::Button | Kind::Toggle | Kind::Item) => {
            vec![format!("focus {h}"), format!("click {h}")]
        }
        _ => vec![format!("focus {h}")],
    }
}
