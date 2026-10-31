//! The line reader: raw mode and the editor while a line is typed.

use std::io::{self, BufRead, Read, Write};

use reva_history::history::History;
use reva_keys::decoder::Decoder;
use reva_lineedit::editor::{Editor, Outcome};
use reva_rawterm::raw;

/// A line from the terminal.
pub enum Input {
    /// A line, with its newline.
    Line(String),
    /// End of input (Ctrl-D on an empty line, or closed stdin).
    Eof,
}

/// Reads lines from stdin, edited and with history when it can.
pub struct LineReader {
    edit: bool,
    pub history: History,
    editor: Editor,
    decoder: Decoder,
}

impl LineReader {
    /// A reader for an interactive stdin; `edit`: try line editing.
    pub fn new(edit: bool) -> Self {
        let dumb = std::env::var("TERM").is_ok_and(|t| t == "dumb");
        let edit = edit && !dumb;
        Self {
            edit,
            history: if edit {
                History::open_default()
            } else {
                History::in_memory()
            },
            editor: Editor::default(),
            decoder: Decoder::default(),
        }
    }

    /// Read a line; `prompt` is what is already shown on the cursor's row
    /// (the program's prompt), redrawn on each change.
    pub fn read_line(&mut self, prompt: &str) -> Input {
        if self.edit
            && let Some(raw) = raw::Raw::enter()
        {
            let r = self.edited(prompt);
            drop(raw);
            if let Input::Line(l) = &r {
                self.history.add(l);
            }
            return r;
        }
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => Input::Eof,
            Ok(_) => Input::Line(line),
        }
    }

    /// The bytes that show `outcome` on the line's row.
    fn draw(&mut self, outcome: &Outcome, prompt: &str) -> String {
        let width = raw::width().or_else(columns).unwrap_or(80);
        match outcome {
            Outcome::Line(_) | Outcome::Eof => {
                // final state, full line, then the newline
                let mut s = self.editor.render(prompt, usize::MAX / 4);
                s.push_str("\r\n");
                s
            }
            Outcome::Cancel => {
                self.editor.reset();
                format!("^C\r\n{}", self.editor.render(prompt, width))
            }
            Outcome::Clear => format!("\x1b[H\x1b[2J{}", self.editor.render(prompt, width)),
            Outcome::Edit => self.editor.render(prompt, width),
        }
    }

    fn edited(&mut self, prompt: &str) -> Input {
        let mut out = io::stdout().lock();
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        self.editor.reset();
        let mut byte = [0u8];
        loop {
            match stdin.read(&mut byte) {
                Ok(1) => {}
                _ => {
                    let _ = out.write_all(b"\r\n");
                    return Input::Eof;
                }
            }
            let Some(k) = self.decoder.push(byte[0]) else {
                continue;
            };
            let outcome = self.editor.key(k, &self.history);
            let draw = self.draw(&outcome, prompt);
            let _ = out.write_all(draw.as_bytes());
            let _ = out.flush();
            match outcome {
                Outcome::Line(mut l) => {
                    l.push('\n');
                    return Input::Line(l);
                }
                Outcome::Eof => return Input::Eof,
                _ => {}
            }
        }
    }
}

/// `$COLUMNS`, when the terminal cannot be asked.
pub fn columns() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()?
        .parse()
        .ok()
        .filter(|&c| c > 0)
}
