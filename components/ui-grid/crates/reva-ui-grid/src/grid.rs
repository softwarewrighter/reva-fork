//! The grid of cells.

/// What a cell is part of; a backend gives each role its theme's colours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Role {
    /// Outside every dialog.
    #[default]
    Screen,
    /// A dialog's or a frame's inside.
    Dialog,
    /// Borders of dialogs, frames and edit panes.
    Border,
    /// A dialog's or a frame's title.
    Title,
    /// Labels.
    Text,
    /// Buttons and toggles.
    Button,
    /// The element with the focus: a button's face.
    Focus,
    /// The border of the edit pane or list with the focus.
    Active,
    /// Where text is edited, lists, canvases.
    Field,
    /// Selected text, a list's selected item.
    Selection,
    /// Elements with ACTIVE=NO.
    Disabled,
}

/// One character cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub role: Role,
}

/// A screen of cells, row by row, and where the text cursor is (the
/// focused text element's caret), if anywhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<Cell>,
    pub cursor: Option<(u16, u16)>,
}
