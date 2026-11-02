//! The cells as terminal output: each role in the terminal's colours, and
//! only the cells that changed since the last screen.

use std::io::Write;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::queue;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use reva_ui_grid::grid::{Grid, Role};

/// A role's colours (foreground, background; `None`: the terminal's own),
/// in the 256-colour palette, close to the page's dark theme.
pub fn colours(role: Role) -> (Option<Color>, Option<Color>) {
    let a = Color::AnsiValue;
    match role {
        Role::Screen => (None, None),
        Role::Dialog | Role::Text => (Some(a(252)), Some(a(236))),
        Role::Border => (Some(a(245)), Some(a(236))),
        Role::Title => (Some(a(79)), Some(a(236))),
        Role::Button => (Some(a(252)), Some(a(239))),
        Role::Focus => (Some(a(234)), Some(a(79))),
        Role::Active => (Some(a(79)), Some(a(236))),
        Role::Field => (Some(a(252)), Some(a(233))),
        Role::Selection => (Some(a(234)), Some(a(72))),
        Role::Disabled => (Some(a(242)), Some(a(236))),
    }
}

/// Write to `out` what turns the terminal showing `prev` (`None`: unknown,
/// so everything) into `next`: runs of changed cells, then the cursor.
pub fn draw(out: &mut impl Write, prev: Option<&Grid>, next: &Grid) -> std::io::Result<()> {
    let same_size = prev.is_some_and(|p| p.cols == next.cols && p.rows == next.rows);
    queue!(out, Hide)?;
    for row in 0..next.rows {
        let mut col = 0;
        while col < next.cols {
            let cell = next.cell(col, row).copied().expect("on the grid");
            if same_size && prev.and_then(|p| p.cell(col, row)) == Some(&cell) {
                col += 1;
                continue;
            }
            let (start, mut run) = (col, String::new());
            while col < next.cols {
                let c = next.cell(col, row).copied().expect("on the grid");
                let unchanged = same_size && prev.and_then(|p| p.cell(col, row)) == Some(&c);
                if c.role != cell.role || (unchanged && !run.is_empty()) {
                    break;
                }
                run.push(c.ch);
                col += 1;
            }
            paint_run(out, start, row, cell.role, &run)?;
        }
    }
    queue!(out, ResetColor, SetAttribute(Attribute::Reset))?;
    if let Some((c, r)) = next.cursor {
        queue!(out, MoveTo(c, r), Show)?;
    }
    out.flush()
}

/// One run of cells of `role` at `col`, `row`.
fn paint_run(
    out: &mut impl Write,
    col: u16,
    row: u16,
    role: Role,
    text: &str,
) -> std::io::Result<()> {
    let (fg, bg) = colours(role);
    queue!(out, MoveTo(col, row), ResetColor)?;
    if let Some(fg) = fg {
        queue!(out, SetForegroundColor(fg))?;
    }
    if let Some(bg) = bg {
        queue!(out, SetBackgroundColor(bg))?;
    }
    let bold = if role == Role::Title {
        Attribute::Bold
    } else {
        Attribute::NormalIntensity
    };
    queue!(out, SetAttribute(bold), Print(text))
}
