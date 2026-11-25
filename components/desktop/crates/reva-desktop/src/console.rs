//! The console on Windows.  The app is a Windows (GUI) program there, so
//! starting it opens no console window (docs/packaging.md, Windows gap 1).
//! A GUI program has no console of its own, so what it prints (`--about`,
//! `--print-output`, errors) would be lost when it is run from cmd or
//! PowerShell: it attaches to the console of the program that started it,
//! if that one has one.  Output redirected to a file or a pipe (Git Bash's
//! `$(...)`, the smoke test) needs nothing: the handles are inherited.

/// Print through the parent's console when standard output is not already
/// a file or a pipe.  Elsewhere, nothing to do.
#[cfg(windows)]
pub fn attach_parent() {
    use windows_sys::Win32::System::Console::{
        ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_OUTPUT_HANDLE,
    };
    // SAFETY: plain Win32 calls with constant arguments; no memory is shared.
    unsafe {
        let out = GetStdHandle(STD_OUTPUT_HANDLE);
        if out.is_null() || out == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

#[cfg(not(windows))]
pub fn attach_parent() {}
