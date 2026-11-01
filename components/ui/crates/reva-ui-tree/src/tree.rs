//! The element tree, and IUP's rules for attributes not set on an
//! element: inherited from the parents, or a default for the kind.

use std::collections::BTreeMap;

use reva_ui_base::change::{Change, Handle};
use reva_ui_base::kind::Kind::{self, *};

/// One element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: Kind,
    pub parent: Option<Handle>,
    pub children: Vec<Handle>,
    /// Attributes set on this element, by IUP's names (upper case).
    pub attrs: BTreeMap<String, String>,
    /// Callbacks by IUP's names (`ACTION`, `K_F6`, `K_ANY`): what the
    /// program gave (for Reva, a Forth word's address).
    pub callbacks: BTreeMap<String, u32>,
}

/// Every element, by handle (handle `h` at index `h - 1`; handles are not
/// reused), and the changes not yet taken.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub nodes: Vec<Option<Node>>,
    pub changes: Vec<Change>,
}

/// The attributes an element takes from its parents when not set on it.
const INHERITED: &[&str] = &["ACTIVE", "BGCOLOR", "FGCOLOR", "FONT", "TABSIZE", "VISIBLE"];

/// Defaults: the kinds they apply to (empty: every kind), name, value.
const DEFAULTS: &[(&[Kind], &str, &str)] = &[
    (
        &[Vbox, Hbox, Zbox, Frame, Radio, Sbox, Tabs, Dialog],
        "EXPAND",
        "YES",
    ),
    (&[], "EXPAND", "NO"),
    (&[Vbox, Zbox], "ALIGNMENT", "ALEFT"),
    (&[Hbox], "ALIGNMENT", "ATOP"),
    (&[Label], "ALIGNMENT", "ALEFT:ACENTER"),
    (&[], "ACTIVE", "YES"),
    (&[], "VISIBLE", "YES"),
    (&[], "GAP", "0"),
    (&[], "MARGIN", "0x0"),
    (&[Toggle], "VALUE", "OFF"),
    (&[Dialog], "TOPMOST", "NO"),
    (&[FileDlg], "DIALOGTYPE", "OPEN"),
    (&[MultiLine], "VISIBLECOLUMNS", "20"),
    (&[MultiLine], "VISIBLELINES", "4"),
    (&[Text], "VISIBLECOLUMNS", "10"),
    (&[Timer], "RUN", "NO"),
    (&[], "TABSIZE", "8"),
];

/// Whether `name` is inherited from the parents.
pub fn inherited(name: &str) -> bool {
    INHERITED.contains(&name)
}

/// The default of attribute `name` for `kind`, if it has one.
pub fn default(kind: Kind, name: &str) -> Option<&'static str> {
    DEFAULTS
        .iter()
        .find(|d| d.1 == name && (d.0.is_empty() || d.0.contains(&kind)))
        .map(|d| d.2)
}

/// IUP's attribute names are case-insensitive: the name as stored.
pub fn normal(name: &str) -> String {
    name.trim().to_ascii_uppercase()
}
