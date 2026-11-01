//! Text and lists inside their panes: a text element's VALUE scrolled to
//! its caret, its selection marked, the cursor there when it has the
//! focus; a list's items, the selected one marked.

use reva_ui_base::change::Handle;
use reva_ui_grid::grid::{Grid, Role};
use reva_ui_layout::geom::Rect;
use reva_ui_mirror::mirror::Mirror;

/// A position `"lin,col"` (or `"col"`) from 1, as (line, column) from 0.
fn parse_pos(s: &str) -> Option<(usize, usize)> {
    let n = |s: &str| s.trim().parse::<usize>().ok().map(|n| n.max(1) - 1);
    match s.split_once(',') {
        Some((l, c)) => Some((n(l)?, n(c)?)),
        None => Some((0, n(s)?)),
    }
}

/// The selection `"pos1:pos2"`, start first.
fn selection(s: &str) -> Option<((usize, usize), (usize, usize))> {
    let (a, b) = s.split_once(':')?;
    let (a, b) = (parse_pos(a)?, parse_pos(b)?);
    Some((a.min(b), a.max(b)))
}

/// Text element `h`'s VALUE in `area`, scrolled to its caret.
pub fn field(g: &mut Grid, m: &Mirror, h: Handle, area: Rect) {
    let t = &m.tree;
    let value = t.attr(h, "VALUE").unwrap_or_default();
    let caret = t
        .attr(h, "CARET")
        .and_then(|c| parse_pos(&c))
        .unwrap_or((0, 0));
    let sel = t.attr(h, "SELECTION").and_then(|s| selection(&s));
    let (rows, cols) = (area.rows.max(1) as usize, area.cols.max(1) as usize);
    let (top, left) = (
        caret.0.saturating_sub(rows - 1),
        caret.1.saturating_sub(cols - 1),
    );
    for (i, line) in value
        .split('\n')
        .enumerate()
        .skip(top)
        .take(area.rows as usize)
    {
        for (j, ch) in line.chars().enumerate().skip(left).take(area.cols as usize) {
            let selected = sel.is_some_and(|(a, b)| (i, j) >= a && (i, j) < b);
            let ch = if ch.is_control() { ' ' } else { ch };
            let role = if selected {
                Role::Selection
            } else {
                Role::Field
            };
            g.put(
                area.col + (j - left) as u16,
                area.row + (i - top) as u16,
                ch,
                role,
            );
        }
    }
    if m.focus == Some(h) && area.cols > 0 && area.rows > 0 {
        g.cursor = Some((
            area.col + (caret.1 - left) as u16,
            area.row + (caret.0 - top) as u16,
        ));
    }
}

/// List `h`'s items (attributes `1`, `2`, ...) in `area`, the one VALUE
/// names selected.
pub fn list(g: &mut Grid, m: &Mirror, h: Handle, area: Rect) {
    let t = &m.tree;
    let selected = t
        .attr(h, "VALUE")
        .and_then(|v| v.trim().parse::<usize>().ok());
    let items = (1..).map_while(|i| t.attr(h, &i.to_string()));
    for (i, item) in items.enumerate().take(area.rows as usize) {
        let role = if selected == Some(i + 1) {
            Role::Selection
        } else {
            Role::Field
        };
        g.text(
            area.col,
            area.row + i as u16,
            &format!("{item:<w$}", w = area.cols as usize),
            role,
            area.cols,
        );
    }
}
