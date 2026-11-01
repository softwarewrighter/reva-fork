//! Sizes and rectangles in character cells.

use reva_ui_base::change::Handle;

/// A size in cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

/// A rectangle in cells, from the dialog's top left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub col: u16,
    pub row: u16,
    pub cols: u16,
    pub rows: u16,
}

/// Where one element goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub handle: Handle,
    pub rect: Rect,
}

/// The directions an element takes extra space in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Expand {
    pub horizontal: bool,
    pub vertical: bool,
}
