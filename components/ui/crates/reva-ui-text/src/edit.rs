//! Editing a [`Text`]: replacing, typing, deleting, moving the caret.

use reva_ui_base::keys::{BS, CR, CTRL, DEL, DOWN, END, HOME, LEFT, RIGHT, SHIFT, UP, printable};

use crate::pos::moved;

/// Emacs keys (as the REPL's line editor has them): Ctrl and a letter, and
/// the key it stands for.  Ctrl-K, kill to the end of the line, is apart.
const EMACS: &[(char, u32)] = &[
    ('A', HOME),
    ('E', END),
    ('B', LEFT),
    ('F', RIGHT),
    ('P', UP),
    ('N', DOWN),
    ('D', DEL),
    ('H', BS),
];
use crate::text::{Motion, Text};

impl Text {
    /// A text holding `value`, the caret at its end.
    pub fn new(value: &str, multiline: bool) -> Text {
        let chars: Vec<char> = value.chars().collect();
        Text {
            caret: chars.len(),
            chars,
            anchor: None,
            multiline,
        }
    }

    /// The selection, start before end; `None` if nothing is selected.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        let (a, b) = (a.min(self.caret), a.max(self.caret));
        (a < b).then_some((a, b))
    }

    /// Select `a`..`b`, the caret at `b`; an empty range selects nothing.
    pub fn select(&mut self, a: usize, b: usize) {
        let len = self.chars.len();
        let (a, b) = (a.min(len), b.min(len));
        self.caret = b;
        self.anchor = (a != b).then_some(a);
    }

    /// Type `s` at the caret, replacing the selection; the caret ends
    /// after it.
    pub fn insert(&mut self, s: &str) {
        let (a, b) = self.selection().unwrap_or((self.caret, self.caret));
        let new: Vec<char> = s.chars().filter(|c| self.multiline || *c != '\n').collect();
        let n = new.len();
        self.chars.splice(a..b, new);
        self.select(a + n, a + n);
    }

    /// Backspace (`forward` false) or Delete: the selection, else one
    /// character before or after the caret.
    pub fn delete(&mut self, forward: bool) {
        if self.selection().is_none() {
            let to = moved(
                &self.chars,
                self.caret,
                if forward { Motion::Right } else { Motion::Left },
            );
            self.select(self.caret, to);
        }
        self.insert("");
    }

    /// Move the caret; with `extend` (Shift held) the selection grows or
    /// shrinks with it, otherwise it goes.
    pub fn move_caret(&mut self, motion: Motion, extend: bool) {
        let anchor = if extend {
            self.anchor.unwrap_or(self.caret)
        } else {
            self.caret
        };
        let to = moved(&self.chars, self.caret, motion);
        self.select(if extend { anchor } else { to }, to);
    }

    /// Edit as key `code` (reva-ui-base's `keys`) does: typing, Backspace,
    /// Delete, Enter in a multiline, the arrows, Home and End (with Ctrl
    /// the ends of the text; with Shift extending the selection).  The
    /// Emacs keys too: Ctrl-A, -E, -B, -F, -P, -N, -D, -H, and Ctrl-K killing
    /// to the end of the line (or joining the next).  False
    /// if the key does not edit.
    pub fn key(&mut self, code: u32) -> bool {
        if code == ('K' as u32 | CTRL) {
            self.move_caret(Motion::LineEnd, true);
            self.delete(true);
            return true;
        }
        let code = EMACS
            .iter()
            .find(|e| code == (e.0 as u32 | CTRL))
            .map_or(code, |e| e.1);
        let (shift, ctrl) = (code & SHIFT != 0, code & CTRL != 0);
        let motion = match code & !(SHIFT | CTRL) {
            LEFT => Some(Motion::Left),
            RIGHT => Some(Motion::Right),
            UP => Some(Motion::Up),
            DOWN => Some(Motion::Down),
            HOME => Some(if ctrl {
                Motion::TextStart
            } else {
                Motion::LineStart
            }),
            END => Some(if ctrl {
                Motion::TextEnd
            } else {
                Motion::LineEnd
            }),
            _ => None,
        };
        match (motion, code) {
            (Some(m), _) => self.move_caret(m, shift),
            (None, BS) => self.delete(false),
            (None, DEL) => self.delete(true),
            (None, CR) if self.multiline => self.insert("\n"),
            _ => match printable(code) {
                Some(c) => self.insert(c.encode_utf8(&mut [0; 4])),
                None => return false,
            },
        }
        true
    }
}
