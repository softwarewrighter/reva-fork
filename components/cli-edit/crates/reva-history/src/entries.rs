//! Reading, adding and searching entries.

use crate::history::{History, MAX};
use std::fs::OpenOptions;
use std::io::Write;

impl History {
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
        if let Some(path) = &self.path
            && let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path)
        {
            let _ = writeln!(f, "{line}");
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
