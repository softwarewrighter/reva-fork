//! Natural sizes and expansion.
//!
//! An element's natural size is what its content needs, drawn as the TUI
//! draws it: a button `[ OK ]`, a toggle `[x] Title`, a text field and a
//! multiline inside a frame of one cell, a dialog and a frame with a
//! border of one cell (the title in the top border).  SIZE or RASTERSIZE
//! replace it, per direction.

use reva_ui_base::change::Handle;
use reva_ui_base::kind::{Kind, Role, role};
use reva_ui_tree::tree::Tree;

use crate::boxes::box_size;
use crate::geom::{Expand, Size};
use crate::units::{expand_attr, user_size};

/// The size of a title: its widest line and its lines; `&` marks a
/// mnemonic and is not shown (`&&` is one `&`).
pub fn title_size(title: &str) -> Size {
    let width = |l: &str| l.replace("&&", "\u{0}").replace('&', "").chars().count();
    let lines: Vec<&str> = title.split('\n').collect();
    let cols = lines.iter().map(|l| width(l)).max().unwrap_or(0);
    Size {
        cols: cols.min(u16::MAX as usize) as u16,
        rows: lines.len() as u16,
    }
}

/// The natural size of `h`, SIZE and RASTERSIZE applied.
pub fn natural(tree: &Tree, h: Handle) -> Size {
    let Some(node) = tree.node(h) else {
        return Size::default();
    };
    let own = own_size(tree, h, node.kind);
    let (cols, rows) = user_size(tree, h);
    Size {
        cols: cols.unwrap_or(own.cols),
        rows: rows.unwrap_or(own.rows),
    }
}

/// The natural size of `h` from its content alone.
fn own_size(tree: &Tree, h: Handle, kind: Kind) -> Size {
    let num = |name, d: u16| {
        tree.attr(h, name)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(d)
    };
    let title = title_size(&tree.attr(h, "TITLE").unwrap_or_default());
    let kids: Vec<Size> = tree.node(h).map_or(vec![], |n| {
        n.children.iter().map(|c| natural(tree, *c)).collect()
    });
    let max = Size {
        cols: kids.iter().map(|s| s.cols).max().unwrap_or(0),
        rows: kids.iter().map(|s| s.rows).max().unwrap_or(0),
    };
    let framed = |s: Size, extra_rows: u16| Size {
        cols: s.cols.saturating_add(2).max(title.cols.saturating_add(4)),
        rows: s.rows.saturating_add(2 + extra_rows),
    };
    match kind {
        Kind::Label => Size {
            rows: title.rows.max(1),
            ..title
        },
        Kind::Button | Kind::Toggle => Size {
            cols: title.cols.saturating_add(4),
            rows: title.rows.max(1),
        },
        Kind::Text => Size {
            cols: num("VISIBLECOLUMNS", 10).saturating_add(2),
            rows: 1,
        },
        Kind::MultiLine => Size {
            cols: num("VISIBLECOLUMNS", 20).saturating_add(2),
            rows: num("VISIBLELINES", 4).saturating_add(2),
        },
        Kind::List => list_size(tree, h),
        Kind::Canvas => Size { cols: 10, rows: 5 },
        Kind::Matrix => Size { cols: 30, rows: 6 },
        Kind::Dialog => framed(max, u16::from(tree.attr(h, "MENU").is_some())),
        Kind::Frame => framed(max, 0),
        Kind::Tabs => framed(max, 1),
        Kind::Vbox | Kind::Hbox => box_size(tree, h, &kids, kind == Kind::Vbox),
        _ if role(kind) == Role::Container => max,
        _ => Size::default(),
    }
}

/// A list: its widest item and its VISIBLELINES (else its items), framed.
fn list_size(tree: &Tree, h: Handle) -> Size {
    let items: Vec<String> = (1..).map_while(|i| tree.attr(h, &i.to_string())).collect();
    let cols = items
        .iter()
        .map(|i| title_size(i).cols)
        .max()
        .unwrap_or(0)
        .max(4);
    let lines = tree
        .attr(h, "VISIBLELINES")
        .and_then(|v| v.parse().ok())
        .unwrap_or(items.len().max(1) as u16);
    Size {
        cols: cols.saturating_add(2),
        rows: lines.saturating_add(2),
    }
}

/// The directions `h` takes extra space in.  A fill expands along its
/// box (unless sized that way); a container only where its EXPAND allows
/// and one of its children expands.
pub fn expands(tree: &Tree, h: Handle) -> Expand {
    let Some(node) = tree.node(h) else {
        return Expand::default();
    };
    let own = expand_attr(tree, h);
    let (cols, rows) = user_size(tree, h);
    match node.kind {
        Kind::Fill => {
            let parent = node.parent.and_then(|p| tree.node(p)).map(|p| p.kind);
            Expand {
                horizontal: parent == Some(Kind::Hbox) && cols.is_none(),
                vertical: parent == Some(Kind::Vbox) && rows.is_none(),
            }
        }
        kind if role(kind) == Role::Container => {
            let any =
                node.children
                    .iter()
                    .map(|c| expands(tree, *c))
                    .fold(Expand::default(), |a, e| Expand {
                        horizontal: a.horizontal || e.horizontal,
                        vertical: a.vertical || e.vertical,
                    });
            Expand {
                horizontal: own.horizontal && any.horizontal,
                vertical: own.vertical && any.vertical,
            }
        }
        _ => own,
    }
}
