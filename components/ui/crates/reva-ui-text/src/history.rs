//! Undo and redo: every edit goes through [`Text::edit`], which remembers
//! the text before it; a run of typing is one step.

use crate::text::Text;

/// The most steps kept.
const DEPTH: usize = 1000;

impl Text {
    /// Put `s` in place of the selection (at the caret if none), the caret
    /// after it, remembering the text before for undo.  `typing`: a typed
    /// character, which joins the step of the typing just before.
    pub fn edit(&mut self, s: &str, typing: bool) {
        let (a, b) = self.selection().unwrap_or((self.caret, self.caret));
        let new: Vec<char> = s.chars().filter(|c| self.multiline || *c != '\n').collect();
        if a == b && new.is_empty() {
            return;
        }
        if !(typing && self.typing) {
            self.undo.push((self.chars.clone(), self.caret));
            if self.undo.len() > DEPTH {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.typing = typing;
        let n = new.len();
        self.chars.splice(a..b, new);
        self.select(a + n, a + n);
    }

    /// Undo the last step; false if there is none.
    pub fn undo(&mut self) -> bool {
        let Some((chars, caret)) = self.undo.pop() else {
            return false;
        };
        self.redo
            .push((std::mem::replace(&mut self.chars, chars), self.caret));
        self.typing = false;
        self.select(caret, caret);
        true
    }

    /// Redo the last step undone; false if there is none.
    pub fn redo(&mut self) -> bool {
        let Some((chars, caret)) = self.redo.pop() else {
            return false;
        };
        self.undo
            .push((std::mem::replace(&mut self.chars, chars), self.caret));
        self.typing = false;
        self.select(caret, caret);
        true
    }
}
