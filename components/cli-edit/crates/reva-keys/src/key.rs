//! A decoded key.

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
