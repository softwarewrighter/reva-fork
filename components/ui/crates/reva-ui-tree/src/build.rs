//! Building the tree: creating, appending, destroying, setting.

use std::collections::BTreeMap;

use reva_ui_base::change::{Change, Handle};
use reva_ui_base::kind::{Kind, Role, role};

use crate::tree::normal;
use crate::tree::{Node, Tree};

impl Tree {
    /// A new element of `kind`, in no parent.
    pub fn create(&mut self, kind: Kind) -> Handle {
        let handle = self.nodes.len() as Handle + 1;
        self.create_at(handle, kind);
        handle
    }

    /// A new element of `kind` at `handle`, where another model made it
    /// (a renderer's copy follows the change stream; reva-ui-mirror),
    /// replacing what was there.
    pub fn create_at(&mut self, handle: Handle, kind: Kind) {
        let Some(i) = (handle as usize).checked_sub(1) else {
            return;
        };
        if self.nodes.len() <= i {
            self.nodes.resize_with(i + 1, || None);
        }
        let (attrs, callbacks) = (BTreeMap::new(), BTreeMap::new());
        self.nodes[i] = Some(Node {
            kind,
            parent: None,
            children: Vec::new(),
            attrs,
            callbacks,
        });
        self.changes.push(Change::Created { handle, kind });
    }

    /// Make `child` the last child of `parent` (taking it out of its old
    /// parent).  False, and nothing done, where IUP refuses: a parent that
    /// holds nothing, a dialog as child, a dialog holding a second child,
    /// or a child that contains `parent`.
    pub fn append(&mut self, parent: Handle, child: Handle) -> bool {
        let (Some(p), Some(c)) = (self.node(parent), self.node(child)) else {
            return false;
        };
        let holds = matches!(role(p.kind), Role::Container | Role::MenuEntry);
        let full = p.kind == Kind::Dialog && !p.children.is_empty();
        if !holds || full || c.kind == Kind::Dialog || self.contains(child, parent) {
            return false;
        }
        self.detach(child);
        self.node_mut(parent).children.push(child);
        self.node_mut(child).parent = Some(parent);
        self.changes.push(Change::Appended { parent, child });
        true
    }

    /// Take `child` out of its parent, if it has one.
    pub fn detach(&mut self, child: Handle) {
        let Some(parent) = self.node(child).and_then(|c| c.parent) else {
            return;
        };
        self.node_mut(parent).children.retain(|c| *c != child);
        self.node_mut(child).parent = None;
        self.changes.push(Change::Detached { child });
    }

    /// Destroy `handle` and everything in it.
    pub fn destroy(&mut self, handle: Handle) {
        let Some(node) = self.node(handle) else {
            return;
        };
        for child in node.children.clone() {
            self.destroy(child);
        }
        self.detach(handle);
        self.nodes[handle as usize - 1] = None;
        self.changes.push(Change::Destroyed { handle });
    }

    /// Set attribute `name` (`None`: remove it, so the inherited or default
    /// value applies again).  False if there is no such element.
    pub fn set(&mut self, handle: Handle, name: &str, value: Option<&str>) -> bool {
        if self.node(handle).is_none() {
            return false;
        }
        let name = normal(name);
        let attrs = &mut self.node_mut(handle).attrs;
        match value {
            Some(v) => attrs.insert(name.clone(), v.to_string()),
            None => attrs.remove(&name),
        };
        self.changes.push(Change::Attr {
            handle,
            name,
            value: value.map(str::to_string),
        });
        true
    }

    /// Set callback `name` to `func` (`None` removes it); the one it
    /// replaces.
    pub fn set_callback(&mut self, handle: Handle, name: &str, func: Option<u32>) -> Option<u32> {
        self.node(handle)?;
        let callbacks = &mut self.node_mut(handle).callbacks;
        match func {
            Some(f) => callbacks.insert(name.to_string(), f),
            None => callbacks.remove(name),
        }
    }
}
