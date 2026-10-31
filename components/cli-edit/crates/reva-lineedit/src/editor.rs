//! The line being edited and what keys do to it.

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
pub struct Search {
    pub back: bool,
    pub query: String,
    /// The history entry shown, if any matched.
    pub at: Option<usize>,
    pub failed: bool,
    pub original: (Vec<char>, usize),
}

/// One line being edited, with history recall and search.
#[derive(Debug, Default)]
pub struct Editor {
    pub(crate) buf: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) kill: Vec<char>,
    /// History entry shown (`None`: the draft).
    pub(crate) index: Option<usize>,
    pub(crate) draft: Vec<char>,
    pub(crate) search: Option<Search>,
    pub(crate) last_query: String,
    /// First char shown, when the line is wider than the terminal.
    pub(crate) scroll: usize,
}

/// A letter, digit or `_`: part of a word.
pub fn is_word(c: char) -> bool {
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

    pub fn set(&mut self, text: &str) {
        self.buf = text.chars().collect();
        self.pos = self.buf.len();
    }

    pub fn cut(&mut self, a: usize, b: usize) {
        if a < b {
            self.kill = self.buf.drain(a..b).collect();
        }
        self.pos = a;
    }
}
