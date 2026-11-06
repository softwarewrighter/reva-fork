//! Drawing into a grid: characters, text, boxes; and the grid as text.

use reva_ui_layout::geom::{Rect, Size};

use crate::grid::{Cell, Grid, Role};

impl Grid {
    /// A grid of `size` filled with blanks of `role`.
    pub fn new(size: Size, role: Role) -> Grid {
        let n = size.cols as usize * size.rows as usize;
        Grid {
            cols: size.cols,
            rows: size.rows,
            cells: vec![Cell { ch: ' ', role }; n],
            canvases: Vec::new(),
            cursor: None,
        }
    }

    /// The cell at `col`, `row`, if it is on the grid.
    pub fn cell(&self, col: u16, row: u16) -> Option<&Cell> {
        (col < self.cols && row < self.rows)
            .then(|| &self.cells[row as usize * self.cols as usize + col as usize])
    }

    /// Put `ch` at `col`, `row` (nothing off the grid).
    pub fn put(&mut self, col: u16, row: u16, ch: char, role: Role) {
        if col < self.cols && row < self.rows {
            self.cells[row as usize * self.cols as usize + col as usize] = Cell { ch, role };
        }
    }

    /// Write `text` from `col`, `row`, at most `max` cells; the cells used.
    pub fn text(&mut self, col: u16, row: u16, text: &str, role: Role, max: u16) -> u16 {
        let mut n = 0;
        for ch in text.chars().filter(|c| !c.is_control()).take(max as usize) {
            self.put(col.saturating_add(n), row, ch, role);
            n += 1;
        }
        n
    }

    /// Fill `r` with `ch`, and with `border` draw a box round it, its
    /// title (if any) in the top border.
    pub fn frame(&mut self, r: Rect, fill: Role, border: Option<(Role, &str, Role)>) {
        for row in r.row..r.row.saturating_add(r.rows) {
            for col in r.col..r.col.saturating_add(r.cols) {
                self.put(col, row, ' ', fill);
            }
        }
        let Some((role, title, title_role)) = border.filter(|_| r.cols >= 2 && r.rows >= 2) else {
            return;
        };
        let (right, bottom) = (r.col + r.cols - 1, r.row + r.rows - 1);
        for col in r.col + 1..right {
            self.put(col, r.row, '─', role);
            self.put(col, bottom, '─', role);
        }
        for row in r.row + 1..bottom {
            self.put(r.col, row, '│', role);
            self.put(right, row, '│', role);
        }
        for (col, row, ch) in [
            (r.col, r.row, '┌'),
            (right, r.row, '┐'),
            (r.col, bottom, '└'),
            (right, bottom, '┘'),
        ] {
            self.put(col, row, ch, role);
        }
        if !title.is_empty() && r.cols > 4 {
            self.text(
                r.col + 1,
                r.row,
                &format!(" {title} "),
                title_role,
                r.cols - 2,
            );
        }
    }

    /// The grid as text, one line a row (trailing blanks dropped), then
    /// `cursor COL,ROW` if it shows: what goldens compare.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for row in self.cells.chunks(self.cols.max(1) as usize) {
            let line: String = row.iter().map(|c| c.ch).collect();
            out.push_str(line.trim_end());
            out.push('\n');
        }
        if let Some((c, r)) = self.cursor {
            out.push_str(&format!("cursor {c},{r}\n"));
        }
        out
    }
}
