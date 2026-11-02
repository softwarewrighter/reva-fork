//! Terminal events as event lines: keys as IUP key codes, clicks.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use reva_ui_base::keys::{BS, CR, DEL, DOWN, END, HOME, LEFT, RIGHT, TAB, UP, code, from_name};

/// The IUP code of a terminal key; `None` for releases and keys IUP has no
/// code for.
pub fn key_code(k: &KeyEvent) -> Option<u32> {
    if k.kind == KeyEventKind::Release {
        return None;
    }
    let m = k.modifiers;
    let (shift, ctrl, alt) = (
        m.contains(KeyModifiers::SHIFT),
        m.contains(KeyModifiers::CONTROL),
        m.contains(KeyModifiers::ALT),
    );
    let base = match k.code {
        KeyCode::Char(c) => c as u32,
        KeyCode::F(n) => from_name(&format!("K_F{n}"))?,
        KeyCode::Enter => CR,
        KeyCode::Tab => TAB,
        KeyCode::BackTab => return Some(code(TAB, true, ctrl, alt)),
        KeyCode::Backspace => BS,
        KeyCode::Delete => DEL,
        KeyCode::Home => HOME,
        KeyCode::End => END,
        KeyCode::Left => LEFT,
        KeyCode::Right => RIGHT,
        KeyCode::Up => UP,
        KeyCode::Down => DOWN,
        KeyCode::PageUp => from_name("K_PGUP")?,
        KeyCode::PageDown => from_name("K_PGDN")?,
        KeyCode::Insert => from_name("K_INS")?,
        KeyCode::Esc => from_name("K_ESC")?,
        _ => return None,
    };
    Some(code(base, shift, ctrl, alt))
}

/// Whether the key leaves the dialog on top as its close box would: Ctrl-C
/// (the terminal has no other way out of a program's loop).
pub fn is_quit(k: &KeyEvent) -> bool {
    k.kind != KeyEventKind::Release
        && k.code == KeyCode::Char('c')
        && k.modifiers.contains(KeyModifiers::CONTROL)
}
