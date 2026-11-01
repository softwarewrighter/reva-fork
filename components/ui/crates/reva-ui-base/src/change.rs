//! The changes a renderer applies to keep its copy of the model.
//!
//! The model records one [`Change`] for everything that alters what is
//! shown; a renderer that applies them in order has the same elements,
//! attributes and visible dialogs (docs/ui-model.md).

use crate::kind::Kind;

/// An element: a small positive integer (0 is IUP's NULL).
pub type Handle = u32;

/// One change to the model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// A new element.
    Created { handle: Handle, kind: Kind },
    /// `child` became the last child of `parent`.
    Appended { parent: Handle, child: Handle },
    /// `child` was taken out of its parent.
    Detached { child: Handle },
    /// The element is gone (its children went first).
    Destroyed { handle: Handle },
    /// An attribute was set (`None`: removed, its default applies).
    Attr {
        handle: Handle,
        name: String,
        value: Option<String>,
    },
    /// A dialog was shown at `x`, `y` (IUP's positions: -1 is centred);
    /// `modal` for `IupPopup`.
    Shown {
        handle: Handle,
        x: i32,
        y: i32,
        modal: bool,
    },
    /// A dialog was hidden.
    Hidden { handle: Handle },
    /// The keyboard focus moved to `handle`.
    Focus { handle: Handle },
}
