//! The text of an element, its caret and its selection.

/// Text being edited.  Positions count characters from 0; the selection
/// runs between `anchor` and the caret.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Text {
    pub chars: Vec<char>,
    pub caret: usize,
    pub anchor: Option<usize>,
    /// IUP's MULTILINE: positions are "lin,col" rather than "col", and
    /// Enter types a newline.
    pub multiline: bool,
}

/// A caret motion (the arrows, Home and End, Ctrl-Home and Ctrl-End).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    TextStart,
    TextEnd,
}
