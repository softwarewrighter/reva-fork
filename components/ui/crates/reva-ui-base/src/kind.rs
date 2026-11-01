//! The kinds of element, by IUP's class names, and their roles.

/// An element's kind: one per IUP constructor Reva's `lib/ui/iup` binds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Dialog,
    Vbox,
    Hbox,
    Zbox,
    Fill,
    Frame,
    Radio,
    Cbox,
    Sbox,
    Tabs,
    Label,
    Button,
    Toggle,
    Text,
    MultiLine,
    List,
    Canvas,
    Matrix,
    Menu,
    Item,
    Submenu,
    Separator,
    FileDlg,
    MessageDlg,
    Timer,
    Image,
    User,
}

/// What an element is for, which decides where it may go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Holds children that are laid out (the dialog holds one).
    Container,
    /// A control: laid out, holds nothing.
    Control,
    /// A menu or menu entry: shown as a menu, never laid out.
    MenuEntry,
    /// Not shown in a dialog (timers, images, file and message dialogs).
    Other,
}

use Kind::*;
use Role::*;

/// Every kind: IUP's class name and its role.
const KINDS: &[(Kind, &str, Role)] = &[
    (Dialog, "dialog", Container),
    (Vbox, "vbox", Container),
    (Hbox, "hbox", Container),
    (Zbox, "zbox", Container),
    (Fill, "fill", Control),
    (Frame, "frame", Container),
    (Radio, "radio", Container),
    (Cbox, "cbox", Container),
    (Sbox, "sbox", Container),
    (Tabs, "tabs", Container),
    (Label, "label", Control),
    (Button, "button", Control),
    (Toggle, "toggle", Control),
    (Text, "text", Control),
    (MultiLine, "multiline", Control),
    (List, "list", Control),
    (Canvas, "canvas", Control),
    (Matrix, "matrix", Control),
    (Menu, "menu", MenuEntry),
    (Item, "item", MenuEntry),
    (Submenu, "submenu", MenuEntry),
    (Separator, "separator", MenuEntry),
    (FileDlg, "filedlg", Other),
    (MessageDlg, "messagedlg", Other),
    (Timer, "timer", Other),
    (Image, "image", Other),
    (User, "user", Other),
];

/// IUP's class name of `kind` (`"multiline"`, `"vbox"`, ...).
pub fn name(kind: Kind) -> &'static str {
    KINDS.iter().find(|k| k.0 == kind).map_or("user", |k| k.1)
}

/// The kind with IUP's class name `name`.
pub fn from_name(name: &str) -> Option<Kind> {
    KINDS.iter().find(|k| k.1 == name).map(|k| k.0)
}

/// The role of `kind`.
pub fn role(kind: Kind) -> Role {
    KINDS.iter().find(|k| k.0 == kind).map_or(Other, |k| k.2)
}

/// Whether `kind` holds editable text (VALUE, CARET, SELECTION, ...).
pub fn is_text(kind: Kind) -> bool {
    matches!(kind, Text | MultiLine)
}

/// Whether `kind` takes the keyboard focus.
pub fn takes_focus(kind: Kind) -> bool {
    matches!(
        kind,
        Text | MultiLine | Button | Toggle | List | Canvas | Matrix
    )
}
