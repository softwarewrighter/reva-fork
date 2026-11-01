//! Applying changes.

use reva_ui_base::change::{Change, Handle};
use reva_ui_base::codec::decode;

use crate::mirror::{Mirror, Shown};

impl Mirror {
    /// Apply one change.
    pub fn apply(&mut self, change: &Change) {
        let t = &mut self.tree;
        match change.clone() {
            Change::Created { handle, kind } => t.create_at(handle, kind),
            Change::Appended { parent, child } => drop(t.append(parent, child)),
            Change::Detached { child } => t.detach(child),
            Change::Destroyed { handle } => {
                t.destroy(handle);
                self.shown.retain(|s| s.handle != handle);
                self.focus = self.focus.filter(|f| *f != handle);
            }
            Change::Attr {
                handle,
                name,
                value,
            } => drop(t.set(handle, &name, value.as_deref())),
            Change::Shown {
                handle,
                x,
                y,
                modal,
            } => {
                self.shown.retain(|s| s.handle != handle);
                self.shown.push(Shown {
                    handle,
                    x,
                    y,
                    modal,
                });
            }
            Change::Hidden { handle } => self.shown.retain(|s| s.handle != handle),
            Change::Focus { handle } => self.focus = Some(handle),
        }
        self.tree.changes.clear();
    }

    /// Apply the changes written as `lines` (reva-ui-base's codec); the
    /// lines that are not changes are skipped.  How many applied.
    pub fn apply_lines(&mut self, lines: &str) -> usize {
        let changes: Vec<Change> = lines.lines().filter_map(decode).collect();
        for c in &changes {
            self.apply(c);
        }
        changes.len()
    }

    /// The dialog on top, if one is shown.
    pub fn top(&self) -> Option<Handle> {
        self.shown.last().map(|s| s.handle)
    }
}
