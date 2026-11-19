//! The copy.

use std::collections::HashMap;

use reva_ui_base::change::Handle;
use reva_ui_tree::tree::Tree;

/// The model as the change stream describes it.
#[derive(Clone, Debug, Default)]
pub struct Mirror {
    pub tree: Tree,
    /// The dialogs shown, the one on top last.
    pub shown: Vec<Shown>,
    pub focus: Option<Handle>,
    /// The renderer's own: the first line and column each text element
    /// was last drawn from (the model has no scroll position), so the view
    /// stays put while the caret is in it, and a click finds the character
    /// it was drawn over.
    pub scroll: HashMap<Handle, (usize, usize)>,
}

/// A dialog shown, where (IUP's position; negative: centred) and whether
/// modally (`IupPopup`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub handle: Handle,
    pub x: i32,
    pub y: i32,
    pub modal: bool,
}
