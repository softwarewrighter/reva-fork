//! The copy.

use reva_ui_base::change::Handle;
use reva_ui_tree::tree::Tree;

/// The model as the change stream describes it.
#[derive(Clone, Debug, Default)]
pub struct Mirror {
    pub tree: Tree,
    /// The dialogs shown, the one on top last.
    pub shown: Vec<Shown>,
    pub focus: Option<Handle>,
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
