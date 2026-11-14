//! Each element as the TUI draws it, in the rectangle the layout gave it:
//! dialogs and frames as boxes titled in their top border (a dialog with
//! its close box `×`), labels as text, buttons `[ OK ]`, toggles
//! `[x] Bold`, text fields between brackets, multilines and lists in a box
//! of their own, canvases blank.  The focused element's face takes the
//! focus role and its border the active one.

use reva_ui_base::change::Handle;
use reva_ui_base::kind::Kind;
use reva_ui_grid::grid::{Grid, Role};
use reva_ui_layout::geom::Rect;
use reva_ui_mirror::mirror::Mirror;

use crate::fields::{field, list};

/// The role `normal` becomes for element `h`: disabled (ACTIVE=NO), the
/// focus role if it has the focus, else itself.
fn role(m: &Mirror, h: Handle, normal: Role) -> Role {
    if m.tree.attr(h, "ACTIVE").is_some_and(|v| v == "NO") {
        Role::Disabled
    } else if m.focus == Some(h) {
        Role::Focus
    } else {
        normal
    }
}

/// Element `h`'s TITLE as shown: `&` marks a mnemonic (`&&` is one `&`).
fn title(m: &Mirror, h: Handle) -> String {
    let t = m.tree.attr(h, "TITLE").unwrap_or_default();
    t.replace("&&", "\u{0}")
        .replace('&', "")
        .replace('\u{0}', "&")
}

/// The columns of the top border of a dialog at `r` that close it: `[×]`
/// and the corner after it, or the `×` alone on a narrow dialog.
pub fn close_cols(r: Rect) -> std::ops::Range<u16> {
    match r.cols {
        0..4 => 0..0,
        4..10 => r.col + r.cols - 2..r.col + r.cols - 1,
        _ => r.col + r.cols - 4..r.col + r.cols,
    }
}

/// A dialog's close box on its top border.
fn close_box(g: &mut Grid, r: Rect) {
    let cols = close_cols(r);
    let marks = if cols.len() > 1 { "[×]" } else { "×" };
    for (c, ch) in cols.zip(marks.chars()) {
        g.put(c, r.row, ch, Role::Title);
    }
}

/// Draw element `h` of `m` at `r`.
pub fn draw(g: &mut Grid, m: &Mirror, h: Handle, r: Rect) {
    let Some(kind) = m.tree.node(h).map(|n| n.kind) else {
        return;
    };
    let title = title(m, h);
    match kind {
        Kind::Dialog | Kind::Frame | Kind::Tabs => {
            g.frame(r, Role::Dialog, Some((Role::Border, &title, Role::Title)));
            if kind == Kind::Dialog {
                close_box(g, r);
            }
        }
        Kind::Label => {
            for (i, line) in title.split('\n').enumerate().take(r.rows as usize) {
                g.text(
                    r.col,
                    r.row + i as u16,
                    line,
                    role(m, h, Role::Text),
                    r.cols,
                );
            }
        }
        Kind::Button | Kind::Toggle => face(g, m, h, kind, &title, r),
        Kind::Text => {
            let border = role(m, h, Role::Border);
            g.text(r.col, r.row, "[", border, 1);
            g.text(r.col + r.cols.saturating_sub(1), r.row, "]", border, 1);
            field(
                g,
                m,
                h,
                Rect {
                    col: r.col + 1,
                    cols: r.cols.saturating_sub(2),
                    rows: 1,
                    ..r
                },
            );
        }
        Kind::MultiLine | Kind::List => pane(g, m, h, kind, r),
        Kind::Canvas => {
            g.frame(r, Role::Field, None);
            g.canvases.push((h, r));
        }
        Kind::Matrix => g.frame(r, Role::Field, None),
        _ => {}
    }
}

/// A button `[ OK ]` (centred in its rectangle) or a toggle `[x] Bold`.
fn face(g: &mut Grid, m: &Mirror, h: Handle, kind: Kind, title: &str, r: Rect) {
    let text = match kind {
        Kind::Toggle => {
            let on = m.tree.attr(h, "VALUE").is_some_and(|v| v == "ON");
            format!("[{}] {title}", if on { 'x' } else { ' ' })
        }
        _ => format!("{:^w$}", format!("[ {title} ]"), w = r.cols as usize),
    };
    g.text(r.col, r.row, &text, role(m, h, Role::Button), r.cols);
}

/// A multiline or a list: a box, its border active when it has the focus,
/// and the text or the items inside.
fn pane(g: &mut Grid, m: &Mirror, h: Handle, kind: Kind, r: Rect) {
    let border = match role(m, h, Role::Border) {
        Role::Focus => Role::Active,
        other => other,
    };
    g.frame(r, Role::Field, Some((border, "", Role::Title)));
    let inside = Rect {
        col: r.col + 1,
        row: r.row + 1,
        cols: r.cols.saturating_sub(2),
        rows: r.rows.saturating_sub(2),
    };
    if kind == Kind::List {
        list(g, m, h, inside)
    } else {
        field(g, m, h, inside)
    }
}
