//! Drawing the line on one terminal row.

use crate::editor::Editor;

impl Editor {
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
