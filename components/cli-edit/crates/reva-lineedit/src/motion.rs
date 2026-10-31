//! Moving over words, and recalling history.

use crate::editor::{Editor, is_word};
use reva_history::history::History;
use reva_keys::key::Key;

impl Editor {
    pub fn word_left(&self) -> usize {
        let mut i = self.pos;
        while i > 0 && !is_word(self.buf[i - 1]) {
            i -= 1;
        }
        while i > 0 && is_word(self.buf[i - 1]) {
            i -= 1;
        }
        i
    }

    pub fn word_right(&self) -> usize {
        let mut i = self.pos;
        while i < self.buf.len() && !is_word(self.buf[i]) {
            i += 1;
        }
        while i < self.buf.len() && is_word(self.buf[i]) {
            i += 1;
        }
        i
    }

    pub fn recall(&mut self, h: &History, to: Option<usize>) {
        if self.index.is_none() {
            self.draft = self.buf.clone();
        }
        self.index = to;
        match to.and_then(|i| h.get(i)) {
            Some(t) => self.set(t),
            None => {
                self.buf = self.draft.clone();
                self.pos = self.buf.len();
            }
        }
    }

    /// A cursor movement; false if `k` is not one.
    pub fn move_key(&mut self, k: Key, n: usize) -> bool {
        match k {
            Key::Ctrl('a') | Key::Home => self.pos = 0,
            Key::Ctrl('e') | Key::End => self.pos = n,
            Key::Ctrl('b') | Key::Left => self.pos = self.pos.saturating_sub(1),
            Key::Ctrl('f') | Key::Right => self.pos = (self.pos + 1).min(n),
            Key::Alt('b') | Key::WordLeft => self.pos = self.word_left(),
            Key::Alt('f') | Key::WordRight => self.pos = self.word_right(),
            _ => return false,
        }
        true
    }
}
