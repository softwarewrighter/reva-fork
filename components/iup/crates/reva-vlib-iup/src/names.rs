//! The C functions `lib/ui/iup` binds, by name: what each does, and the
//! address `(func)` gives it.
//!
//! The functions are not gates: their addresses are entries of this table
//! in the null page (`FN_BASE`), which no code runs from, and `(call)`
//! looks them up here.  An image stores the addresses, so the table is
//! append-only, like the gates.

use reva_ui_base::kind::Kind;

/// Handle `(lib)` returns for the library ("IUP").
pub const IUP_HANDLE: u32 = 0x4955_5000;

/// The names Reva loads IUP by, on each OS (`lib/ui/iup-common`).
pub const IUP_NAMES: &[&str] = &[
    "libiupall.so",
    "libiup.dylib",
    "libiupcd.dylib",
    "libiupcontrols.dylib",
    "libiupgl.dylib",
    "iupall.dll",
];

/// Where the functions' addresses start: inside the null guard, so that
/// jumping to one faults rather than running anything.
pub const FN_BASE: u32 = 0x0000_0400;

/// What a new element takes from the call's arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Args {
    /// Nothing (or an action name, which Reva does not use).
    None,
    /// The first is its TITLE (a C string, or NULL).
    Title,
    /// Every argument is a child (NULL ones skipped).
    Children,
    /// The first is its TITLE, the second its child.
    TitleChild,
    /// The first points to a NULL-terminated array of children.
    Array,
}

/// What a function does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// Returns `0` (IUP_NOERROR): opening, closing, mapping, refreshing,
    /// and what the model does not need.
    Zero,
    /// Returns `-2` (IUP_DEFAULT): `IupLoopStep`.
    Default,
    /// Returns a constant.
    Value(i32),
    /// A new element.
    New(Kind, Args),
    Show,
    ShowXY,
    Popup,
    Hide,
    Destroy,
    MainLoop,
    Append,
    SetAttr,
    SetAttrId,
    GetAttr,
    GetAttrId,
    SetCallback,
    NextChild,
    DialogChild,
    Parent,
    Dialog,
    KeyName,
    KeyCode,
    Version,
    /// `IupExitLoop`: the innermost loop ends once the running callback
    /// returns.
    ExitLoop,
}

use Args as A;
use Kind as K;
use Op::*;

/// Every function: name, what it does (the arguments are as many as
/// `(call)` passes: `lib/ui/iup` binds `IupTabs` with 1 to 10).  Append only.
pub const FUNCS: &[(&str, Op)] = &[
    ("IupOpen", Zero),
    ("IupClose", Zero),
    ("IupLabel", New(K::Label, A::Title)),
    ("IupDialog", New(K::Dialog, A::Children)),
    ("IupShow", Show),
    ("IupMap", Zero),
    ("IupHide", Hide),
    ("IupDestroy", Destroy),
    ("IupPopup", Popup),
    ("IupShowXY", ShowXY),
    ("IupMainLoop", MainLoop),
    ("IupLoopStep", Default),
    ("IupFlush", Zero),
    ("IupRefresh", Zero),
    ("IupUpdate", Zero),
    ("IupVbox", New(K::Vbox, A::Children)),
    ("IupHbox", New(K::Hbox, A::Children)),
    ("IupZbox", New(K::Zbox, A::Children)),
    ("IupAppend", Append),
    ("IupButton", New(K::Button, A::Title)),
    ("IupSetFunction", Zero),
    ("IupSetAttribute", SetAttr),
    ("IupStoreAttribute", SetAttr),
    ("IupStoreAttributeId", SetAttrId),
    ("IupGetAttribute", GetAttr),
    ("IupGetAttributeId", GetAttrId),
    ("IupFill", New(K::Fill, A::None)),
    ("IupRadio", New(K::Radio, A::Children)),
    ("IupFrame", New(K::Frame, A::Children)),
    ("IupList", New(K::List, A::None)),
    ("IupText", New(K::Text, A::None)),
    ("IupMultiLine", New(K::MultiLine, A::None)),
    ("IupToggle", New(K::Toggle, A::Title)),
    ("IupFileDlg", New(K::FileDlg, A::None)),
    ("IupTimer", New(K::Timer, A::None)),
    ("IupGetActionName", Zero),
    ("IupCanvas", New(K::Canvas, A::None)),
    ("IupSetCallback", SetCallback),
    ("IupUser", New(K::User, A::None)),
    ("IupSbox", New(K::Sbox, A::Children)),
    ("IupImage", New(K::Image, A::None)),
    ("IupSetHandle", Zero),
    ("IupMenuv", New(K::Menu, A::Array)),
    ("IupMenu", New(K::Menu, A::Children)),
    ("IupItem", New(K::Item, A::Title)),
    ("IupSeparator", New(K::Separator, A::None)),
    ("IupSubmenu", New(K::Submenu, A::TitleChild)),
    ("IupGetNextChild", NextChild),
    ("IupGetDialogChild", DialogChild),
    ("IupHelp", Zero),
    ("IupLoadImage", Zero),
    ("IupGetParent", Parent),
    ("IupGetDialog", Dialog),
    ("IupTabs", New(K::Tabs, A::Children)),
    ("iupKeyCodeToName", KeyName),
    ("iupKeyNameToCode", KeyCode),
    ("IupVersion", Version),
    ("iupdrvGetScrollbarSize", Value(16)),
    ("IupControlsOpen", Zero),
    ("IupControlsClose", Zero),
    ("IupDial", New(K::User, A::None)),
    ("IupMatrix", New(K::Matrix, A::None)),
    ("IupMatStoreAttribute", Zero),
    ("IupMatGetAttribute", Zero),
    ("IupMatGetInt", Zero),
    ("IupMatGetFloat", Zero),
    ("IupCbox", New(K::Cbox, A::Children)),
    ("IupMessageDlg", New(K::MessageDlg, A::None)),
    ("IupGLCanvasOpen", Zero),
    ("IupGLCanvas", New(K::Canvas, A::None)),
    ("IupGLMakeCurrent", Zero),
    ("IupGLIsCurrent", Zero),
    ("IupGLSwapBuffers", Zero),
    ("IupGLPalette", Zero),
    ("IupExitLoop", ExitLoop),
];

/// The address `(func)` gives function `name`.
pub fn lookup(name: &str) -> Option<u32> {
    let i = FUNCS.iter().position(|f| f.0 == name)?;
    Some(FN_BASE + 4 * i as u32)
}

/// The function at address `addr`: its name and what it does.
pub fn function(addr: u32) -> Option<(&'static str, Op)> {
    let off = addr.checked_sub(FN_BASE)?;
    let f = FUNCS.get((off / 4) as usize).filter(|_| off % 4 == 0)?;
    Some((f.0, f.1))
}
