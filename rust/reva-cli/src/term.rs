//! Reading a line at a terminal with [`lineedit`](crate::lineedit): raw
//! mode while the line is edited, the terminal's own mode while Forth runs.
//!
//! Raw mode is switched on only inside [`LineReader::read_line`], so
//! program output, `bye`, a crash or Ctrl-C while a word runs all happen
//! with the terminal as it was.  The VM still reads whole lines: editing
//! happens before the line reaches it, as with a terminal in canonical
//! mode, so `KEY` behaves as before (it gets the next character of the
//! line).
//!
//! No crates: Unix uses `tcgetattr`/`cfmakeraw`/`tcsetattr` from the C
//! library, through an opaque buffer so the code does not depend on any
//! platform's `struct termios` layout; Windows uses `SetConsoleMode` with
//! virtual terminal input.  If raw mode cannot be had (an odd terminal,
//! Git Bash's mintty pipes, `TERM=dumb`, `--no-edit`) the reader falls
//! back to the plain cooked `read_line` the CLI always used.

use std::io::{self, BufRead, Read, Write};

use crate::history::History;
use crate::lineedit::{Decoder, Editor, Outcome};

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
        if self.edit {
            if let Some(raw) = raw::Raw::enter() {
                let r = self.edited(prompt);
                drop(raw);
                if let Input::Line(l) = &r {
                    self.history.add(l);
                }
                return r;
            }
        }
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => Input::Eof,
            Ok(_) => Input::Line(line),
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
            let width = raw::width().unwrap_or(80);
            let draw = match &outcome {
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
            };
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

#[cfg(unix)]
mod raw {
    use std::os::raw::{c_int, c_ulong};
    use std::sync::Mutex;

    /// Big enough and aligned for any platform's `struct termios`
    /// (Linux: 60 bytes, macOS: 72, the BSDs: 44).
    type Termios = [u64; 32];

    unsafe extern "C" {
        fn tcgetattr(fd: c_int, t: *mut Termios) -> c_int;
        fn tcsetattr(fd: c_int, action: c_int, t: *const Termios) -> c_int;
        fn cfmakeraw(t: *mut Termios);
        #[allow(dead_code)]
        fn ioctl(fd: c_int, req: c_ulong, ...) -> c_int;
    }
    const TCSANOW: c_int = 0;

    /// The mode to put back, for the panic hook.
    static SAVED: Mutex<Option<Termios>> = Mutex::new(None);

    /// Raw mode on stdin until dropped.
    pub struct Raw(Termios);

    impl Raw {
        pub fn enter() -> Option<Raw> {
            let mut t: Termios = [0; 32];
            // SAFETY: `t` is larger than any termios; fd 0 is stdin.
            unsafe {
                if tcgetattr(0, &mut t) != 0 {
                    return None;
                }
                let saved = t;
                cfmakeraw(&mut t);
                if tcsetattr(0, TCSANOW, &t) != 0 {
                    return None;
                }
                install_hook();
                *SAVED.lock().ok()? = Some(saved);
                Some(Raw(saved))
            }
        }
    }

    impl Drop for Raw {
        fn drop(&mut self) {
            // SAFETY: restores the mode tcgetattr gave us.
            unsafe { tcsetattr(0, TCSANOW, &self.0) };
            if let Ok(mut s) = SAVED.lock() {
                *s = None;
            }
        }
    }

    /// Release builds abort on panic without unwinding, so Drop would not
    /// run: put the terminal back from the hook.
    fn install_hook() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let prev = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                if let Ok(Some(t)) = SAVED.try_lock().map(|s| *s) {
                    // SAFETY: as in Drop.
                    unsafe { tcsetattr(0, TCSANOW, &t) };
                }
                prev(info);
            }));
        });
    }

    // Linux: 0x5413, except on MIPS, PowerPC and SPARC, which use the BSD
    // encoding.  Should the ioctl fail, $COLUMNS (or 80) is used.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TIOCGWINSZ: c_ulong = if cfg!(any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "sparc64"
    )) {
        0x4008_7468
    } else {
        0x5413
    };
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    const TIOCGWINSZ: c_ulong = 0x4008_7468;

    /// Terminal columns.
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    pub fn width() -> Option<usize> {
        let mut ws = [0u16; 4]; // rows, cols, xpixel, ypixel
        // SAFETY: TIOCGWINSZ fills a struct winsize (four u16s).
        let ok = unsafe { ioctl(1, TIOCGWINSZ, ws.as_mut_ptr()) } == 0;
        (ok && ws[1] > 0)
            .then_some(ws[1] as usize)
            .or_else(super::columns)
    }

    /// Terminal columns: `$COLUMNS` (no TIOCGWINSZ known here).
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    pub fn width() -> Option<usize> {
        super::columns()
    }
}

#[cfg(windows)]
mod raw {
    use std::ffi::c_void;

    type Handle = *mut c_void;
    const STD_INPUT_HANDLE: u32 = -10i32 as u32;
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_PROCESSED_INPUT: u32 = 0x1;
    const ENABLE_LINE_INPUT: u32 = 0x2;
    const ENABLE_ECHO_INPUT: u32 = 0x4;
    const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x200;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;

    #[repr(C)]
    #[derive(Default)]
    struct BufferInfo {
        size: [i16; 2],
        cursor: [i16; 2],
        attributes: u16,
        window: [i16; 4], // left, top, right, bottom
        max: [i16; 2],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(n: u32) -> Handle;
        fn GetConsoleMode(h: Handle, mode: *mut u32) -> i32;
        fn SetConsoleMode(h: Handle, mode: u32) -> i32;
        fn GetConsoleScreenBufferInfo(h: Handle, info: *mut BufferInfo) -> i32;
    }

    /// Raw (VT input) mode on the console until dropped.
    pub struct Raw {
        input: u32,
        output: u32,
    }

    impl Raw {
        pub fn enter() -> Option<Raw> {
            // SAFETY: plain console API calls on the process's handles.
            unsafe {
                let (i, o) = (
                    GetStdHandle(STD_INPUT_HANDLE),
                    GetStdHandle(STD_OUTPUT_HANDLE),
                );
                let (mut input, mut output) = (0, 0);
                if GetConsoleMode(i, &mut input) == 0 || GetConsoleMode(o, &mut output) == 0 {
                    return None;
                }
                let raw_in = (input
                    & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
                    | ENABLE_VIRTUAL_TERMINAL_INPUT;
                if SetConsoleMode(i, raw_in) == 0 {
                    return None;
                }
                if SetConsoleMode(o, output | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0 {
                    SetConsoleMode(i, input);
                    return None;
                }
                Some(Raw { input, output })
            }
        }
    }

    impl Drop for Raw {
        fn drop(&mut self) {
            // SAFETY: restores the modes read in `enter`.
            unsafe {
                SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), self.input);
                SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), self.output);
            }
        }
    }

    /// Console columns.
    pub fn width() -> Option<usize> {
        let mut info = BufferInfo::default();
        // SAFETY: fills the struct it is given.
        let ok =
            unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) } != 0;
        let w = (info.window[2] - info.window[0] + 1) as usize;
        (ok && w > 0).then_some(w).or_else(super::columns)
    }
}

#[cfg(not(any(unix, windows)))]
mod raw {
    pub struct Raw;
    impl Raw {
        pub fn enter() -> Option<Raw> {
            None
        }
    }
    pub fn width() -> Option<usize> {
        super::columns()
    }
}

/// `$COLUMNS`, when the terminal cannot be asked.
#[allow(dead_code)]
fn columns() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()?
        .parse()
        .ok()
        .filter(|&c| c > 0)
}
