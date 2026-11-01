//! Placing a dialog's elements.
//!
//! A box lays its children out one after another (a vbox down, an hbox
//! across) at their natural sizes, then shares the space left equally
//! among the children that expand along it.  Across, a child that expands
//! takes the box's whole width (or height); the others go where the box's
//! ALIGNMENT puts them.  Other containers give each child their inside:
//! all of it where the child expands, its natural size elsewhere.

use reva_ui_base::change::Handle;
use reva_ui_base::kind::{Kind, Role, role};
use reva_ui_tree::tree::Tree;

use crate::boxes::boxed;
use crate::geom::{Placed, Rect, Size};
use crate::natural::{expands, natural};
use crate::units::user_size;

/// Where every element of `dialog` goes, the dialog first, in a window of
/// `avail` cells: the dialog at its SIZE, else its natural size, at most
/// `avail`.
pub fn layout(tree: &Tree, dialog: Handle, avail: Size) -> Vec<Placed> {
    let n = natural(tree, dialog);
    let (cols, rows) = user_size(tree, dialog);
    let cols = cols.unwrap_or(n.cols).min(avail.cols);
    let rows = rows.unwrap_or(n.rows).min(avail.rows);
    let mut out = Vec::new();
    place(
        tree,
        dialog,
        Rect {
            col: 0,
            row: 0,
            cols,
            rows,
        },
        &mut out,
    );
    out
}

/// Place `h` at `rect`, and what is in it.
pub(crate) fn place(tree: &Tree, h: Handle, rect: Rect, out: &mut Vec<Placed>) {
    out.push(Placed { handle: h, rect });
    let Some(node) = tree.node(h) else {
        return;
    };
    let kids: Vec<Handle> = node
        .children
        .iter()
        .copied()
        .filter(|c| laid_out(tree, *c))
        .collect();
    match node.kind {
        Kind::Vbox | Kind::Hbox => boxed(tree, h, &kids, rect, node.kind == Kind::Vbox, out),
        kind if role(kind) == Role::Container => {
            let inside = inside(tree, h, kind, rect);
            for c in kids {
                let (n, e) = (natural(tree, c), expands(tree, c));
                let cols = if e.horizontal {
                    inside.cols
                } else {
                    n.cols.min(inside.cols)
                };
                let rows = if e.vertical {
                    inside.rows
                } else {
                    n.rows.min(inside.rows)
                };
                place(
                    tree,
                    c,
                    Rect {
                        cols,
                        rows,
                        ..inside
                    },
                    out,
                );
            }
        }
        _ => {}
    }
}

/// Whether `h` takes part in the layout (menus, timers and the like do not).
fn laid_out(tree: &Tree, h: Handle) -> bool {
    tree.node(h)
        .is_some_and(|n| matches!(role(n.kind), Role::Container | Role::Control))
}

/// The inside of a container's border: a dialog's and a frame's one cell
/// (and a dialog's menu bar), tabs' border and tab row.
fn inside(tree: &Tree, h: Handle, kind: Kind, r: Rect) -> Rect {
    let menu = u16::from(tree.attr(h, "MENU").is_some());
    let (top, side, bottom) = match kind {
        Kind::Dialog => (1 + menu, 1, 1),
        Kind::Frame => (1, 1, 1),
        Kind::Tabs => (2, 1, 1),
        _ => (0, 0, 0),
    };
    Rect {
        col: r.col.saturating_add(side),
        row: r.row.saturating_add(top),
        cols: r.cols.saturating_sub(2 * side),
        rows: r.rows.saturating_sub(top + bottom),
    }
}

/// `r` less `margin` cells on each side.
pub(crate) fn inset(r: Rect, margin: Size) -> Rect {
    Rect {
        col: r.col.saturating_add(margin.cols),
        row: r.row.saturating_add(margin.rows),
        cols: r.cols.saturating_sub(2 * margin.cols),
        rows: r.rows.saturating_sub(2 * margin.rows),
    }
}
