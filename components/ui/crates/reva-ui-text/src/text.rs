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
    /// The texts (and carets) before each edit, the last on top, for undo;
    /// and those undone, for redo.
    pub undo: Vec<(Vec<char>, usize)>,
    pub redo: Vec<(Vec<char>, usize)>,
    /// Whether the last edit was typing, so typing goes on in one step.
    pub typing: bool,
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
