//! REPL history: the lines typed at the prompt, oldest first, kept in a
//! plain text file (one entry per line) so they survive `bye`, crashes and
//! the `run-cosy` restart loop.
//!
//! Where: `$REVA_HISTORY` if set (empty: keep history in memory only),
//! else `~/.reva_history`.  A package's launcher picks its own file
//! (`run-cosy` uses `~/.cosy_history`), so packages keep their histories
//! apart, as the web page does with its per-package storage key.
//!
//! Each accepted line is appended to the file at once (no rewrite, so two
//! sessions at a time lose nothing; their lines interleave).  The file is
//! rewritten only when loading finds it at more than twice [`MAX`]
//! entries, to keep it bounded.  Blank lines and a repeat of the last entry
//! are not recorded.  Errors are ignored: history is a convenience and must
//! never stop the REPL.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Entries kept (and loaded).
pub const MAX: usize = 1000;

/// The history list, and the file it is kept in.
#[derive(Debug, Default)]
pub struct History {
    entries: Vec<String>,
    path: Option<PathBuf>,
}

impl History {
    /// An empty history kept in memory only (tests, `REVA_HISTORY=`).
    pub fn in_memory() -> Self {
        Self::default()
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

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Entry `i` (0 is the oldest).
    pub fn get(&self, i: usize) -> Option<&str> {
        self.entries.get(i).map(String::as_str)
    }

    /// Record `line` (without its newline), unless it is blank or repeats
    /// the last entry.
    pub fn add(&mut self, line: &str) {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.trim().is_empty() || self.entries.last().map(String::as_str) == Some(line) {
            return;
        }
        self.entries.push(line.to_string());
        if self.entries.len() > MAX {
            self.entries.remove(0);
        }
        if let Some(path) = &self.path {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(f, "{line}");
            }
        }
    }

    /// The newest entry before `from` (exclusive) containing `query`.
    pub fn search_back(&self, query: &str, from: usize) -> Option<usize> {
        (0..from.min(self.entries.len()))
            .rev()
            .find(|&i| self.entries[i].contains(query))
    }

    /// The oldest entry after `from` (exclusive) containing `query`.
    pub fn search_forward(&self, query: &str, from: usize) -> Option<usize> {
        (from + 1..self.entries.len()).find(|&i| self.entries[i].contains(query))
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_blank_and_repeats() {
        let mut h = History::in_memory();
        for l in ["1 .", "", "  ", "1 .", "2 .", "1 ."] {
            h.add(l);
        }
        assert_eq!(h.entries, ["1 .", "2 .", "1 ."]);
    }

    #[test]
    fn searches_both_ways() {
        let mut h = History::in_memory();
        for l in ["dup", "swap", "dup dup", "drop"] {
            h.add(l);
        }
        assert_eq!(h.search_back("dup", 4), Some(2));
        assert_eq!(h.search_back("dup", 2), Some(0));
        assert_eq!(h.search_back("dup", 0), None);
        assert_eq!(h.search_forward("dup", 0), Some(2));
        assert_eq!(h.search_forward("dup", 2), None);
    }

    #[test]
    fn file_round_trip_and_trim() {
        let dir = std::env::temp_dir().join(format!("reva-hist-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("h");
        let many: String = (0..2 * MAX + 5).map(|i| format!("{i} .\n")).collect();
        fs::write(&path, many).unwrap();
        let mut h = History::open(path.clone());
        assert_eq!(h.len(), MAX);
        assert_eq!(h.get(MAX - 1), Some(&*format!("{} .", 2 * MAX + 4)));
        h.add("words");
        let again = History::open(path.clone());
        assert_eq!(again.get(again.len() - 1), Some("words"));
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), MAX + 1);
        let _ = fs::remove_dir_all(dir);
    }
}
