//! Open a URL in the user's browser, with the tool each OS provides (no
//! crate needed): `open` on macOS, `xdg-open` elsewhere on Unix, and
//! `explorer` on Windows.  `$BROWSER`, when set, wins everywhere.

use std::process::{Command, Stdio};

/// The command that would open `url` here.
pub fn command(url: &str) -> Command {
    let mut c = match std::env::var_os("BROWSER").filter(|b| !b.is_empty()) {
        Some(b) => Command::new(b),
        None if cfg!(target_os = "macos") => Command::new("open"),
        // explorer.exe treats a URL argument as "open with the default
        // browser" and, unlike `cmd /c start`, needs no quoting of `&`.
        None if cfg!(windows) => Command::new("explorer"),
        None => Command::new("xdg-open"),
    };
    c.arg(url);
    c
}

/// Open `url`; the error says what to do by hand.
pub fn browser(url: &str) -> Result<(), String> {
    let mut c = command(url);
    let prog = c.get_program().to_string_lossy().into_owned();
    c.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        // reap it when it exits (xdg-open returns at once; open too)
        .map(|mut child| drop(std::thread::spawn(move || child.wait())))
        .map_err(|e| format!("could not run {prog} ({e}); open {url} in a browser"))
}
