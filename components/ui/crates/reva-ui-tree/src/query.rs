//! Reading the tree: elements, attributes, changes.

use reva_ui_base::change::{Change, Handle};

use crate::tree::{Node, Tree};
use crate::tree::{default, inherited, normal};

impl Tree {
    /// The element `handle`, if it exists.
    pub fn node(&self, handle: Handle) -> Option<&Node> {
        let i = (handle as usize).checked_sub(1)?;
        self.nodes.get(i)?.as_ref()
    }

    /// The element `handle`, which must exist.
    pub(crate) fn node_mut(&mut self, handle: Handle) -> &mut Node {
        self.nodes[handle as usize - 1]
            .as_mut()
            .expect("a live element")
    }

    /// Attribute `name` of `handle` as IUP reads it: set on the element,
    /// else (if inherited) on the nearest parent that has it, else the
    /// default for the element's kind.
    pub fn attr(&self, handle: Handle, name: &str) -> Option<String> {
        let name = normal(name);
        let node = self.node(handle)?;
        let mut at = Some(node);
        while let Some(n) = at {
            if let Some(v) = n.attrs.get(&name) {
                return Some(v.clone());
            }
            at = n
                .parent
                .filter(|_| inherited(&name))
                .and_then(|p| self.node(p));
        }
        default(node.kind, &name).map(str::to_string)
    }

    /// The changes since the last call, oldest first.
    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }
}
