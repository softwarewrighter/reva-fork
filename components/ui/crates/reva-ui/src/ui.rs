//! The model, its events, and the callbacks they run.

use std::collections::BTreeMap;

use reva_ui_base::change::Handle;
use reva_ui_text::text::Text;
use reva_ui_tree::tree::Tree;

/// What a callback returns (IUP's `IUP_IGNORE` ... `IUP_CONTINUE`; Reva's
/// `gui-ignore` ... `gui-continue`).
pub const IGNORE: i32 = -1;
pub const DEFAULT: i32 = -2;
pub const CLOSE: i32 = -3;
pub const CONTINUE: i32 = -4;

/// The UI of one program: its elements, the text of its text elements,
/// the dialogs shown, the keyboard focus.
#[derive(Clone, Debug, Default)]
pub struct Ui {
    pub tree: Tree,
    pub texts: BTreeMap<Handle, Text>,
    /// The dialogs shown, the one on top last.
    pub shown: Vec<Handle>,
    pub focus: Option<Handle>,
    /// A callback returned [`CLOSE`]: the main loop ends.
    pub closed: bool,
}

/// Something the user did, as a renderer reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A key (reva-ui-base's `keys` codes) for the focused element.
    Key(u32),
    /// A button, toggle or menu item clicked.
    Click(Handle),
    /// The focus moved to an element (a click into it).
    Focus(Handle),
    /// A dialog's close box.
    Close(Handle),
}

/// A callback the host runs for an event: function `func` (for Reva, a
/// Forth word) of element `handle`, given the handle and `args`.  The
/// host then gives its result to `Ui::finish`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub handle: Handle,
    pub name: String,
    pub func: u32,
    pub args: Vec<i32>,
    pub event: Event,
}
