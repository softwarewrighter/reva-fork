//! Relations: parents, children, the dialog, names, callbacks.

use reva_ui_base::change::Handle;

use crate::tree::Tree;

impl Tree {
    /// Whether `inner` is `outer` or inside it.
    pub fn contains(&self, outer: Handle, inner: Handle) -> bool {
        self.dialog_path(inner).contains(&outer)
    }

    /// `handle`, its parent, and so on to the top.
    pub fn dialog_path(&self, handle: Handle) -> Vec<Handle> {
        let mut path = Vec::new();
        let mut at = self.node(handle).map(|_| handle);
        while let Some(h) = at {
            path.push(h);
            at = self.node(h).and_then(|n| n.parent);
        }
        path
    }

    /// The child of `parent` after `child` (`None`: the first), as IUP's
    /// `IupGetNextChild`.
    pub fn next_child(&self, parent: Handle, child: Option<Handle>) -> Option<Handle> {
        let children = &self.node(parent)?.children;
        match child {
            None => children.first().copied(),
            Some(c) => children.iter().skip_while(|h| **h != c).nth(1).copied(),
        }
    }

    /// The element in `root` (itself included) whose NAME is `name`, as
    /// IUP's `IupGetDialogChild`; depth first.
    pub fn named(&self, root: Handle, name: &str) -> Option<Handle> {
        let node = self.node(root)?;
        if node.attrs.get("NAME").is_some_and(|n| n == name) {
            return Some(root);
        }
        node.children.iter().find_map(|c| self.named(*c, name))
    }

    /// Callback `name` of `handle` or, failing that, of the nearest parent
    /// that has it (IUP looks up key callbacks this way): the element it
    /// was found on, and the function.
    pub fn callback(&self, handle: Handle, name: &str) -> Option<(Handle, u32)> {
        let path = self.dialog_path(handle);
        path.into_iter()
            .find_map(|h| Some((h, *self.node(h)?.callbacks.get(name)?)))
    }
}
