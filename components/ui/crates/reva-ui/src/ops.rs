//! Creating, appending, destroying, showing and hiding elements.

use reva_ui_base::change::{Change, Handle};
use reva_ui_base::kind::{Kind, is_text, takes_focus};
use reva_ui_text::text::Text;

use crate::ui::Ui;

impl Ui {
    /// An empty model.
    pub fn new() -> Ui {
        Ui::default()
    }

    /// A new element of `kind` (IUP's `IupLabel`, `IupMultiLine`, ...).
    pub fn create(&mut self, kind: Kind) -> Handle {
        let handle = self.tree.create(kind);
        if is_text(kind) {
            self.texts
                .insert(handle, Text::new("", kind == Kind::MultiLine));
        }
        handle
    }

    /// Destroy `handle` and everything in it (`IupDestroy`).
    pub fn destroy(&mut self, handle: Handle) {
        let gone: Vec<Handle> = self
            .texts
            .keys()
            .copied()
            .filter(|h| self.tree.contains(handle, *h))
            .collect();
        for h in gone {
            self.texts.remove(&h);
        }
        if self.focus.is_some_and(|f| self.tree.contains(handle, f)) {
            self.focus = None;
        }
        self.shown.retain(|d| *d != handle);
        self.tree.destroy(handle);
    }

    /// Show dialog `handle` on top at `x`, `y` (`IupShowXY`; `modal` for
    /// `IupPopup`), giving the focus to its first element that takes it.
    pub fn show(&mut self, handle: Handle, x: i32, y: i32, modal: bool) -> bool {
        if self
            .tree
            .node(handle)
            .is_none_or(|n| n.kind != Kind::Dialog)
        {
            return false;
        }
        self.shown.retain(|d| *d != handle);
        self.shown.push(handle);
        self.tree.changes.push(Change::Shown {
            handle,
            x,
            y,
            modal,
        });
        if !self.focus.is_some_and(|f| self.tree.contains(handle, f)) {
            self.focus = self.focusable(handle).first().copied();
            if let Some(f) = self.focus {
                self.tree.changes.push(Change::Focus { handle: f });
            }
        }
        true
    }

    /// Hide dialog `handle` (`IupHide`).
    pub fn hide(&mut self, handle: Handle) {
        if self.shown.contains(&handle) {
            self.shown.retain(|d| *d != handle);
            self.tree.changes.push(Change::Hidden { handle });
        }
    }

    /// Whether `IupMainLoop` goes on: a dialog is shown and no callback
    /// returned `CLOSE`.
    pub fn running(&self) -> bool {
        !self.closed && !self.shown.is_empty()
    }

    /// The elements in `root` that take the focus, in order: active,
    /// visible, depth first.
    pub fn focusable(&self, root: Handle) -> Vec<Handle> {
        let Some(node) = self.tree.node(root) else {
            return Vec::new();
        };
        let on = |name| self.tree.attr(root, name).is_none_or(|v| v != "NO");
        if !on("ACTIVE") || !on("VISIBLE") {
            return Vec::new();
        }
        let mut out: Vec<Handle> = if takes_focus(node.kind) {
            vec![root]
        } else {
            Vec::new()
        };
        for c in &node.children {
            out.extend(self.focusable(*c));
        }
        out
    }
}
