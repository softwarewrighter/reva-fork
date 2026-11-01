//! The folder as files for the page, within size limits.

use crate::paths::to_latin1;
use std::fs;
use std::path::Path;

/// Files above this size are not loaded into the VM at start (it keeps all
/// files in memory); a note says which were skipped.
pub const MAX_FILE: u64 = 16 << 20;
/// Nor more than this in total.
pub const MAX_TOTAL: u64 = 256 << 20;

/// Every file under `root`, as (VM path, bytes), within the size limits.
pub fn snapshot(root: &Path, notes: &mut Vec<String>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut total = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let Ok(meta) = e.metadata() else { continue };
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue; // .git and friends
            }
            if meta.is_dir() {
                stack.push(p);
                continue;
            }
            let Ok(rel) = p.strip_prefix(root) else {
                continue;
            };
            let vm = format!("/{}", rel.to_string_lossy().replace('\\', "/"));
            if meta.len() > MAX_FILE || total + meta.len() > MAX_TOTAL {
                notes.push(format!("not loaded (too large): {vm}"));
                continue;
            }
            if let Ok(bytes) = fs::read(&p) {
                total += bytes.len() as u64;
                out.push((vm, to_latin1(&bytes)));
            }
        }
    }
    out
}
