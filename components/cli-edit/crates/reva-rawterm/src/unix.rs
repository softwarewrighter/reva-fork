//! Raw mode on Unix: `tcgetattr`/`cfmakeraw`/`tcsetattr` through an
//! opaque buffer, and the width from `TIOCGWINSZ`.

use std::os::raw::c_int;
use std::sync::Mutex;

/// Big enough and aligned for any platform's `struct termios`
/// (Linux: 60 bytes, macOS: 72, the BSDs: 44).
type Termios = [u64; 32];

unsafe extern "C" {
    fn tcgetattr(fd: c_int, t: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, action: c_int, t: *const Termios) -> c_int;
    fn cfmakeraw(t: *mut Termios);
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
