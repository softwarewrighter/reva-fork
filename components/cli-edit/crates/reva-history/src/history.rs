//! The history and its file.

use std::fs;
use std::path::PathBuf;

/// Entries kept (and loaded).
pub const MAX: usize = 1000;

/// The history list, and the file it is kept in.
#[derive(Debug, Default)]
pub struct History {
    pub entries: Vec<String>,
    pub path: Option<PathBuf>,
}

impl History {
    /// An empty history kept in memory only (tests, `REVA_HISTORY=`).
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// No entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The history at the default place (see the module docs), loaded.
    pub fn open_default() -> Self {
        let path = match std::env::var_os("REVA_HISTORY") {
            Some(p) if p.is_empty() => None,
            Some(p) => Some(PathBuf::from(p)),
            None => home().map(|h| h.join(".reva_history")),
        };
        match path {
            Some(p) => Self::open(p),
            None => Self::in_memory(),
        }
    }

    /// The history kept in `path`, loaded (a missing file is empty).
    pub fn open(path: PathBuf) -> Self {
        let text = fs::read_to_string(&path).unwrap_or_default();
        let all: Vec<String> = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect();
        let entries = all[all.len().saturating_sub(MAX)..].to_vec();
        if all.len() > 2 * MAX {
            let mut keep = entries.join("\n");
            keep.push('\n');
            let _ = fs::write(&path, keep);
        }
        Self {
            entries,
            path: Some(path),
        }
    }
}

/// The home directory: `$HOME`, or `%USERPROFILE%`.
pub fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}
