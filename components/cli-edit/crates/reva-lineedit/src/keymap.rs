//! What keys do: the line's ends, edits, history and search.

use crate::editor::{Editor, Outcome, Search};
use reva_history::history::History;
use reva_keys::key::Key;

impl Editor {
    /// Apply `k`.
    pub fn key(&mut self, k: Key, h: &History) -> Outcome {
        if self.search.is_some()
            && let Some(o) = self.search_key(k, h)
        {
            return o;
        }
        let n = self.buf.len();
        match k {
            Key::Enter => return Outcome::Line(self.text()),
            Key::Ctrl('c') => return Outcome::Cancel,
            Key::Ctrl('d') if n == 0 => return Outcome::Eof,
            Key::Ctrl('l') => return Outcome::Clear,
            _ => {}
        }
        if !self.move_key(k, n) && !self.edit_key(k, n) {
            self.history_key(k, h);
        }
        Outcome::Edit
    }

    /// An edit of the text; false if `k` is not one.
    pub fn edit_key(&mut self, k: Key, n: usize) -> bool {
        match k {
            Key::Char(c) => {
                self.buf.insert(self.pos, c);
                self.pos += 1;
            }
            Key::Ctrl('d') | Key::Delete if self.pos < n => {
                self.buf.remove(self.pos);
            }
            Key::Ctrl('h') | Key::Backspace if self.pos > 0 => {
                self.pos -= 1;
                self.buf.remove(self.pos);
            }
            Key::Ctrl('d') | Key::Delete | Key::Ctrl('h') | Key::Backspace => {}
            Key::Ctrl('k') => self.cut(self.pos, n),
            Key::Ctrl('u') => self.cut(0, self.pos),
            Key::Ctrl('w') | Key::AltBackspace => self.cut(self.word_left(), self.pos),
            Key::Alt('d') => self.cut(self.pos, self.word_right()),
            Key::Ctrl('y') => {
                let k = self.kill.clone();
                self.buf.splice(self.pos..self.pos, k.iter().copied());
                self.pos += k.len();
            }
            Key::Ctrl('t') if n >= 2 && self.pos > 0 => {
                let p = if self.pos == n { n - 1 } else { self.pos };
                self.buf.swap(p - 1, p);
                self.pos = p + 1;
            }
            _ => return false,
        }
        true
    }

    /// History recall (Up, Down) and the start of a search (Ctrl-R, Ctrl-S).
    pub fn history_key(&mut self, k: Key, h: &History) {
        match k {
            Key::Ctrl('p') | Key::Up => {
                let to = self.index.unwrap_or(h.len()).checked_sub(1);
                if let Some(i) = to {
                    self.recall(h, Some(i));
                }
            }
            Key::Ctrl('n') | Key::Down => {
                if let Some(i) = self.index {
                    self.recall(h, (i + 1 < h.len()).then_some(i + 1));
                }
            }
            Key::Ctrl('r') | Key::Ctrl('s') => {
                self.search = Some(Search {
                    back: k == Key::Ctrl('r'),
                    query: String::new(),
                    at: None,
                    failed: false,
                    original: (self.buf.clone(), self.pos),
                });
            }
            _ => {}
        }
    }

    /// A key while searching: `Some` if it was the search's, `None` if it
    /// ended the search and should now be applied to the line.
    pub fn search_key(&mut self, k: Key, h: &History) -> Option<Outcome> {
        let s = self.search.as_mut().unwrap();
        match k {
            Key::Ctrl('r') | Key::Ctrl('s') => {
                s.back = k == Key::Ctrl('r');
                if s.query.is_empty() {
                    s.query = self.last_query.clone();
                }
                let from = s.at;
                self.find(h, from, false);
            }
            Key::Char(c) => {
                s.query.push(c);
                let from = s.at;
                self.find(h, from, true);
            }
            Key::Backspace => {
                s.query.pop();
                s.at = None;
                self.find(h, None, true);
            }
            Key::Ctrl('g') | Key::Ctrl('c') | Key::Esc => {
                let s = self.search.take().unwrap();
                self.last_query = s.query;
                (self.buf, self.pos) = s.original;
            }
            _ => {
                let s = self.search.take().unwrap();
                if !s.query.is_empty() {
                    self.last_query = s.query;
                }
                self.index = s.at;
                return None;
            }
        }
        Some(Outcome::Edit)
    }

    /// Look for the query from history entry `from`; `stay`: the entry at
    /// `from` itself may match (the query grew).
    pub fn find(&mut self, h: &History, from: Option<usize>, stay: bool) {
        let s = self.search.as_mut().unwrap();
        let found = if s.query.is_empty() {
            None
        } else if s.back {
            let start = from.map_or(h.len(), |i| if stay { i + 1 } else { i });
            h.search_back(&s.query, start)
        } else {
            match from {
                Some(i) if stay && h.get(i).is_some_and(|t| t.contains(&s.query)) => Some(i),
                Some(i) => h.search_forward(&s.query, i),
                None => (0..h.len()).find(|&i| h.get(i).is_some_and(|t| t.contains(&s.query))),
            }
        };
        s.failed = found.is_none() && !s.query.is_empty();
        if let Some(i) = found {
            s.at = Some(i);
            let text = h.get(i).unwrap_or("");
            let q = s.query.clone();
            self.set(text);
            if let Some(b) = text.find(&q) {
                self.pos = text[..b].chars().count();
            }
        }
    }
}
