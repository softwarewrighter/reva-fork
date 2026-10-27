//! The terminal line editor's logic: key decoding, editing, history
//! recall, Ctrl-R search and drawing.  No I/O here (see `term.rs`), so all
//! of it is unit tested.
//!
//! The keys are the web page's (`www/lineedit.js`); `docs/repl.md` has
//! the table.  The line is drawn on one terminal row after the prompt,
//! scrolling sideways when it is wider than the terminal, so a redraw is
//! always "carriage return, prompt, visible text, clear to end of row,
//! move to the cursor" and never depends on where wrapped rows went.

use crate::history::History;

/// A decoded key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    /// Ctrl plus a letter (`'a'..='z'`), except the ones below.
    Ctrl(char),
    /// Alt (Meta, Esc prefix) plus a character.
    Alt(char),
    Enter,
    Backspace,
    AltBackspace,
    Delete,
    Left,
    Right,
    WordLeft,
    WordRight,
    Home,
    End,
    Up,
    Down,
    Esc,
    /// A sequence this editor does not use.
    Ignored,
}

/// Turns terminal input bytes into [`Key`]s: UTF-8, control bytes, and the
/// `ESC [`/`ESC O` sequences xterm-like terminals (and Windows consoles
/// with virtual terminal input) send.  A lone Esc followed by a character
/// is Alt plus that character, as in readline, so no timing is needed.
#[derive(Debug, Default)]
pub struct Decoder {
    pending: Vec<u8>,
    after_cr: bool,
}

impl Decoder {
    /// Feed one byte; returns a key once one is complete.
    pub fn push(&mut self, b: u8) -> Option<Key> {
        let after_cr = std::mem::take(&mut self.after_cr);
        if self.pending.is_empty() {
            return match b {
                b'\r' => {
                    self.after_cr = true;
                    Some(Key::Enter)
                }
                b'\n' if after_cr => None, // CR LF (pasted text): one Enter
                b'\n' => Some(Key::Enter),
                0x1b => {
                    self.pending.push(b);
                    None
                }
                8 | 0x7f => Some(Key::Backspace),
                b'\t' => Some(Key::Char(' ')),
                1..=26 => Some(Key::Ctrl((b'a' + b - 1) as char)),
                0..=31 => Some(Key::Ignored),
                0x20..=0x7e => Some(Key::Char(b as char)),
                _ => {
                    self.pending.push(b);
                    self.utf8()
                }
            };
        }
        self.pending.push(b);
        if self.pending[0] == 0x1b {
            self.escape()
        } else {
            self.utf8()
        }
    }

    fn utf8(&mut self) -> Option<Key> {
        let need = match self.pending[0] {
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            _ => 1,
        };
        if self.pending.len() < need {
            return None;
        }
        let k = std::str::from_utf8(&self.pending)
            .ok()
            .and_then(|s| s.chars().next())
            .map_or(Key::Ignored, Key::Char);
        self.pending.clear();
        Some(k)
    }

    fn escape(&mut self) -> Option<Key> {
        let p = &self.pending;
        if p.len() == 2 {
            return match p[1] {
                b'[' | b'O' => None,
                0x7f | 8 => self.done(Key::AltBackspace),
                0x1b => self.done(Key::Esc),
                c if c.is_ascii_graphic() => self.done(Key::Alt(c.to_ascii_lowercase() as char)),
                _ => self.done(Key::Esc),
            };
        }
        let last = *p.last().unwrap();
        if !(0x40..=0x7e).contains(&last) {
            // parameters and intermediates; give up on runaway sequences
            return if p.len() > 16 {
                self.done(Key::Ignored)
            } else {
                None
            };
        }
        let params = String::from_utf8_lossy(&p[2..p.len() - 1]).into_owned();
        let modified = params.ends_with(";5") || params.ends_with(";3");
        let k = match (last, params.as_str()) {
            (b'A', _) => Key::Up,
            (b'B', _) => Key::Down,
            (b'C', _) if modified => Key::WordRight,
            (b'D', _) if modified => Key::WordLeft,
            (b'C', _) => Key::Right,
            (b'D', _) => Key::Left,
            (b'H', _) => Key::Home,
            (b'F', _) => Key::End,
            (b'~', "1" | "7") => Key::Home,
            (b'~', "4" | "8") => Key::End,
            (b'~', "3") => Key::Delete,
            _ => Key::Ignored,
        };
        self.done(k)
    }

    fn done(&mut self, k: Key) -> Option<Key> {
        self.pending.clear();
        Some(k)
    }
}

/// What a key did to the line.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Still editing: redraw.
    Edit,
    /// Enter: the finished line (without newline).
    Line(String),
    /// Ctrl-C: drop the line, start a new one.
    Cancel,
    /// Ctrl-D on an empty line: end of input.
    Eof,
    /// Ctrl-L: clear the screen, then redraw.
    Clear,
}

#[derive(Debug)]
struct Search {
    back: bool,
    query: String,
    /// The history entry shown, if any matched.
    at: Option<usize>,
    failed: bool,
    original: (Vec<char>, usize),
}

/// One line being edited, with history recall and search.
#[derive(Debug, Default)]
pub struct Editor {
    buf: Vec<char>,
    pos: usize,
    kill: Vec<char>,
    /// History entry shown (`None`: the draft).
    index: Option<usize>,
    draft: Vec<char>,
    search: Option<Search>,
    last_query: String,
    /// First char shown, when the line is wider than the terminal.
    scroll: usize,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Editor {
    /// Start a new, empty line (the kill buffer and last search are kept).
    pub fn reset(&mut self) {
        let kill = std::mem::take(&mut self.kill);
        let last_query = std::mem::take(&mut self.last_query);
        *self = Editor {
            kill,
            last_query,
            ..Editor::default()
        };
    }

    /// The text being edited.
    pub fn text(&self) -> String {
        self.buf.iter().collect()
    }

    fn set(&mut self, text: &str) {
        self.buf = text.chars().collect();
        self.pos = self.buf.len();
    }

    fn word_left(&self) -> usize {
        let mut i = self.pos;
        while i > 0 && !is_word(self.buf[i - 1]) {
            i -= 1;
        }
        while i > 0 && is_word(self.buf[i - 1]) {
            i -= 1;
        }
        i
    }

    fn word_right(&self) -> usize {
        let mut i = self.pos;
        while i < self.buf.len() && !is_word(self.buf[i]) {
            i += 1;
        }
        while i < self.buf.len() && is_word(self.buf[i]) {
            i += 1;
        }
        i
    }

    fn cut(&mut self, a: usize, b: usize) {
        if a < b {
            self.kill = self.buf.drain(a..b).collect();
        }
        self.pos = a;
    }

    fn recall(&mut self, h: &History, to: Option<usize>) {
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

    /// Apply `k`.
    pub fn key(&mut self, k: Key, h: &History) -> Outcome {
        if self.search.is_some()
            && let Some(o) = self.search_key(k, h)
        {
            return o;
        }
        let n = self.buf.len();
        match k {
            Key::Char(c) => {
                self.buf.insert(self.pos, c);
                self.pos += 1;
            }
            Key::Enter => return Outcome::Line(self.text()),
            Key::Ctrl('c') => return Outcome::Cancel,
            Key::Ctrl('d') if n == 0 => return Outcome::Eof,
            Key::Ctrl('l') => return Outcome::Clear,
            Key::Ctrl('a') | Key::Home => self.pos = 0,
            Key::Ctrl('e') | Key::End => self.pos = n,
            Key::Ctrl('b') | Key::Left => self.pos = self.pos.saturating_sub(1),
            Key::Ctrl('f') | Key::Right => self.pos = (self.pos + 1).min(n),
            Key::Alt('b') | Key::WordLeft => self.pos = self.word_left(),
            Key::Alt('f') | Key::WordRight => self.pos = self.word_right(),
            Key::Ctrl('d') | Key::Delete => {
                if self.pos < n {
                    self.buf.remove(self.pos);
                }
            }
            Key::Ctrl('h') | Key::Backspace => {
                if self.pos > 0 {
                    self.pos -= 1;
                    self.buf.remove(self.pos);
                }
            }
            Key::Ctrl('k') => self.cut(self.pos, n),
            Key::Ctrl('u') => self.cut(0, self.pos),
            Key::Ctrl('w') | Key::AltBackspace => self.cut(self.word_left(), self.pos),
            Key::Alt('d') => {
                let (p, e) = (self.pos, self.word_right());
                self.cut(p, e);
            }
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
        Outcome::Edit
    }

    /// A key while searching: `Some` if it was the search's, `None` if it
    /// ended the search and should now be applied to the line.
    fn search_key(&mut self, k: Key, h: &History) -> Option<Outcome> {
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
    fn find(&mut self, h: &History, from: Option<usize>, stay: bool) {
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

    /// The bytes that redraw the line on the current row: `prompt` (what
    /// the program printed last on this row), then the line, scrolled to
    /// fit `width` columns, with the cursor placed.
    pub fn render(&mut self, prompt: &str, width: usize) -> String {
        let shown;
        let prompt = match &self.search {
            Some(s) => {
                shown = format!(
                    "({}{}-i-search)`{}': ",
                    if s.failed { "failed " } else { "" },
                    if s.back { "reverse" } else { "forward" },
                    s.query
                );
                shown.as_str()
            }
            None => prompt,
        };
        let pw = visible_width(prompt);
        let room = width.saturating_sub(pw + 1).max(8);
        if self.pos < self.scroll {
            self.scroll = self.pos;
        } else if self.pos > self.scroll + room {
            self.scroll = self.pos - room;
        }
        if self.buf.len() <= room {
            self.scroll = 0;
        }
        let end = (self.scroll + room).min(self.buf.len());
        let text: String = self.buf[self.scroll..end].iter().collect();
        let col = pw + self.pos - self.scroll;
        let mut out = format!("\r{prompt}{text}\x1b[K\r");
        if col > 0 {
            out.push_str(&format!("\x1b[{col}C"));
        }
        out
    }
}

/// Columns `s` takes, leaving out escape sequences (one per char otherwise).
pub fn visible_width(s: &str) -> usize {
    let mut n = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else if !c.is_control() {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(bytes: &[u8]) -> Vec<Key> {
        let mut d = Decoder::default();
        bytes.iter().filter_map(|&b| d.push(b)).collect()
    }

    fn typed(e: &mut Editor, h: &History, bytes: &[u8]) -> Vec<Outcome> {
        keys(bytes)
            .into_iter()
            .map(|k| e.key(k, h))
            .filter(|o| *o != Outcome::Edit)
            .collect()
    }

    fn hist(lines: &[&str]) -> History {
        let mut h = History::in_memory();
        for l in lines {
            h.add(l);
        }
        h
    }

    #[test]
    fn decodes_keys() {
        use Key::*;
        assert_eq!(
            keys(b"a\x01\x7f\r\n\x1b[A\x1b[D\x1bOH\x1b[3~\x1b[1;5C\x1bb\x1b\x7f\x1b[200~"),
            [
                Char('a'),
                Ctrl('a'),
                Backspace,
                Enter,
                Up,
                Left,
                Home,
                Delete,
                WordRight,
                Alt('b'),
                AltBackspace,
                Ignored
            ]
        );
        assert_eq!(keys("é€".as_bytes()), [Char('é'), Char('€')]);
    }

    #[test]
    fn edits() {
        let h = History::in_memory();
        let mut e = Editor::default();
        typed(&mut e, &h, b"dup swap drop\x1bb\x0b\x01\x19");
        assert_eq!(e.text(), "dropdup swap ");
        typed(&mut e, &h, b"\x05\x17");
        assert_eq!(e.text(), "dropdup ");
        typed(&mut e, &h, b"\x15ab\x14");
        assert_eq!(e.text(), "ba");
        assert_eq!(typed(&mut e, &h, b"\r"), [Outcome::Line("ba".into())]);
    }

    #[test]
    fn ctrl_d_and_c() {
        let h = History::in_memory();
        let mut e = Editor::default();
        assert_eq!(typed(&mut e, &h, b"ab\x02\x04"), []);
        assert_eq!(e.text(), "a");
        assert_eq!(typed(&mut e, &h, b"\x03"), [Outcome::Cancel]);
        e.reset();
        assert_eq!(typed(&mut e, &h, b"\x04"), [Outcome::Eof]);
    }

    #[test]
    fn history_recall_keeps_draft() {
        let h = hist(&["1 .", "2 ."]);
        let mut e = Editor::default();
        typed(&mut e, &h, b"dr\x1b[A");
        assert_eq!(e.text(), "2 .");
        typed(&mut e, &h, b"\x10\x10");
        assert_eq!(e.text(), "1 .");
        typed(&mut e, &h, b"\x1b[B\x0e");
        assert_eq!(e.text(), "dr");
    }

    #[test]
    fn reverse_search() {
        let h = hist(&["1 2 + .", ": sq dup * ;", "7 sq ."]);
        let mut e = Editor::default();
        typed(&mut e, &h, b"xy\x12sq");
        assert_eq!(e.text(), "7 sq .");
        assert!(e.render("ok> ", 80).contains("(reverse-i-search)`sq': "));
        typed(&mut e, &h, b"\x12");
        assert_eq!(e.text(), ": sq dup * ;");
        typed(&mut e, &h, b"\x12");
        assert!(e.render("ok> ", 80).contains("failed"));
        typed(&mut e, &h, b"\x07");
        assert_eq!(e.text(), "xy");
        // Ctrl-R Ctrl-R repeats the last search; Ctrl-E keeps the match.
        typed(&mut e, &h, b"\x12\x12\x05");
        assert_eq!(e.text(), "7 sq .");
        assert_eq!(e.render("ok> ", 80), "\rok> 7 sq .\x1b[K\r\x1b[10C");
        assert_eq!(
            typed(&mut e, &h, b"\x12+\r"),
            [Outcome::Line("1 2 + .".into())]
        );
    }

    #[test]
    fn scrolls_long_lines() {
        let h = History::in_memory();
        let mut e = Editor::default();
        typed(&mut e, &h, "x".repeat(50).as_bytes());
        let r = e.render("ok> ", 20);
        assert_eq!(r, format!("\rok> {}\x1b[K\r\x1b[19C", "x".repeat(15)));
        typed(&mut e, &h, b"\x01");
        assert!(e.render("ok> ", 20).ends_with("\r\x1b[4C"));
        assert_eq!(visible_width("\x1b[1mok>\x1b[0m "), 4);
    }
}
