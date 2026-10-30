//! Changing the tree: stamps, copies of embedded files, renames.

use crate::tree::MemTree;
use reva_embedded::embedded;

impl MemTree {
    /// Record a write to `p`.
    pub fn touch(&mut self, p: &str) {
        self.changed.insert(p.to_string());
        self.mtimes.insert(p.to_string(), self.time);
    }

    /// Make an embedded file writable by copying it.
    pub fn materialize(&mut self, p: &str) {
        if !self.files.contains_key(p) {
            let data = embedded::lib_file(p)
                .map(<[u8]>::to_vec)
                .unwrap_or_default();
            self.files.insert(p.to_string(), data);
        }
    }

    /// Move the file `a` to `b` (both absolute, checked by the caller).
    pub fn rename_file(&mut self, a: &str, b: &str) {
        let data = self.files.remove(a).unwrap_or_default();
        self.files.insert(b.to_string(), data);
        self.mtimes.remove(a);
        self.changed.insert(a.to_string());
        self.touch(b);
    }

    /// Move the directory `a`, and everything below it, to `b`.
    pub fn rename_dir(&mut self, a: &str, b: &str) {
        let prefix = format!("{a}/");
        let moved: Vec<String> = self
            .files
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in moved {
            let data = self.files.remove(&k).unwrap_or_default();
            let nk = format!("{b}{}", &k[a.len()..]);
            self.files.insert(nk.clone(), data);
            self.changed.insert(k);
            self.touch(&nk);
        }
        let dirs: Vec<String> = self
            .dirs
            .iter()
            .filter(|k| **k == a || k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in dirs {
            self.dirs.remove(&k);
            self.dirs.insert(format!("{b}{}", &k[a.len()..]));
        }
    }
}
