//! Answering the `ui` host calls in the terminal.

use std::io::{Write, stdout};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use reva_hostcall::hostcall::{HostAnswer, HostCall, HostReply};
use reva_ui_base::keys::pasted;
use reva_ui_layout::geom::Size;
use reva_ui_paint::paint::{click, paint};

use crate::keys::{is_quit, key_code};
use crate::screen::draw;
use crate::term::UiTerm;

/// The terminal's size in cells.
fn size() -> Size {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    Size { cols, rows }
}

impl UiTerm {
    /// The answer to `call`: a `wait` applies the changes, draws, and
    /// returns the next event; a file dialog is cancelled for now.
    pub fn answer(&mut self, call: &HostCall) -> HostAnswer {
        match call.op {
            "wait" => {
                let text = call
                    .bytes
                    .first()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .unwrap_or_default();
                self.mirror.apply_lines(&text);
                match self.next_event() {
                    Ok(line) => HostAnswer::Reply(HostReply::Bytes(line.into_bytes())),
                    Err(e) => {
                        self.suspend();
                        HostAnswer::Unavailable(format!("the terminal UI failed: {e}"))
                    }
                }
            }
            "file" => HostAnswer::Reply(HostReply::Int(-1)),
            op => HostAnswer::Unavailable(format!("ui {op} is not available in the terminal")),
        }
    }

    /// Draw, then wait for a key or click that makes an event line.
    fn next_event(&mut self) -> std::io::Result<String> {
        self.enter()?;
        loop {
            if let Some(line) = self.pending.pop_front() {
                return Ok(line);
            }
            let screen = size();
            let grid = paint(&self.mirror, screen);
            draw(&mut stdout(), self.shown.as_ref(), &grid)?;
            self.shown = Some(grid);
            match event::read()? {
                Event::Key(k) if is_quit(&k) => {
                    let top = self.mirror.top().unwrap_or(0);
                    self.pending.push_back(format!("close {top}"));
                }
                Event::Key(k) => self
                    .pending
                    .extend(key_code(&k).map(|c| format!("key {c}"))),
                Event::Mouse(m) if m.kind == MouseEventKind::Down(MouseButton::Left) => {
                    self.pending
                        .extend(click(&self.mirror, screen, m.column, m.row));
                }
                // Terminals paste line breaks as \r; one Enter each.
                Event::Paste(text) => self.pending.extend(pasted(&text)),
                Event::Resize(..) => self.shown = None,
                _ => {}
            }
        }
    }

    /// Into TUI mode: raw input, the alternate screen, mouse reports; a
    /// panic puts the terminal back first (the release build aborts).
    fn enter(&mut self) -> std::io::Result<()> {
        if self.active {
            return Ok(());
        }
        static HOOK: std::sync::Once = std::sync::Once::new();
        HOOK.call_once(|| {
            let hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                restore();
                hook(info);
            }));
        });
        terminal::enable_raw_mode()?;
        execute!(
            stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            event::EnableBracketedPaste
        )?;
        // Terminals that can (kitty, WezTerm, Ghostty, iTerm2 with CSI u)
        // tell Ctrl-Enter from Enter; elsewhere Ctrl-J or F6 evaluate.
        if terminal::supports_keyboard_enhancement().unwrap_or(false) {
            let flags = event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
            execute!(stdout(), event::PushKeyboardEnhancementFlags(flags))?;
            ENHANCED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.active = true;
        self.shown = None;
        Ok(())
    }

    /// Back to the REPL's terminal (before it prompts, at exit, on errors).
    pub fn suspend(&mut self) {
        if std::mem::take(&mut self.active) {
            restore();
        }
    }
}

/// Leave TUI mode, whatever state the terminal is in.
/// Whether the keyboard enhancement is on (to switch it off again).
static ENHANCED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn restore() {
    let mut out = stdout();
    if ENHANCED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        let _ = execute!(out, event::PopKeyboardEnhancementFlags);
    }
    let _ = execute!(
        out,
        event::DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen,
        crossterm::cursor::Show
    );
    let _ = terminal::disable_raw_mode();
    let _ = out.flush();
}

impl Drop for UiTerm {
    fn drop(&mut self) {
        self.suspend();
    }
}
