//! The decoder.

use crate::key::Key;

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
