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
    /// A callback returned [`CLOSE`]: the innermost loop ends.
    pub closed: bool,
    /// The event loops running, innermost last (`IupMainLoop` inside
    /// `IupPopup` inside ...).
    pub loops: Vec<Loop>,
    /// The callbacks running, innermost last, each with what the host
    /// restores when it returns (for Reva, the data stack: ESI and EAX),
    /// or after a fault in it (the return stack at its entry, and the
    /// catch handler then).
    pub calls: Vec<(Call, [u32; 4])>,
    /// CD's canvases on the IUP canvases (reva-cd); their drawings reach
    /// the renderer as the canvases' DRAW attribute.
    pub cd: reva_cd::state::Cd,
}

/// An event loop: `IupMainLoop`, or `IupPopup` of a dialog (it ends when
/// the dialog is hidden), or `IupLoopStep`: one event, then it ends (Reva
/// 7.0's `gui-main-loop` calls it in a loop); `true` once it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loop {
    Main,
    Popup(Handle),
    Step(bool),
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
    /// The renderer laid a canvas out at a width and height (pixels):
    /// RESIZE_CB, then ACTION (the redraw).
    Size(Handle, i32, i32),
    /// A running timer's TIME passed: its ACTION_CB.
    Timer(Handle),
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
