//! A script of events, one per line:
//!
//! ```text
//! # a comment
//! key K_F6          a key, by IUP's name or code
//! type 2 3 +        the characters, one key each
//! click 5           a button, toggle or menu item (by handle)
//! focus 3
//! close 1           a dialog's close box
//! file /tmp/a.txt   the answer to a file dialog
//! cancel            a file dialog cancelled
//! ```

use std::collections::VecDeque;

/// The events not yet given, and what the host showed so far.
#[derive(Clone, Debug, Default)]
pub struct UiScript {
    pub lines: VecDeque<String>,
}

/// The script `text` as lines of events: comments and blank lines left
/// out, `type` made one `key` per character.
pub fn lines(text: &str) -> VecDeque<String> {
    let mut out = VecDeque::new();
    for line in text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        match line.strip_prefix("type ") {
            Some(chars) => out.extend(chars.chars().map(|c| format!("key {}", c as u32))),
            None => out.push_back(line.trim().to_string()),
        }
    }
    out
}
