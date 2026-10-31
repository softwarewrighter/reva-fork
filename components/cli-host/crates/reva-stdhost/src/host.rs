//! The host's state.

use std::io::{self, Write};

use reva_stdfs::stdfs::StdFs;
use reva_term::reader::{Input, LineReader};

/// The command-line host.
pub struct StdHost {
    pub out: io::BufWriter<io::Stdout>,
    pub fs: StdFs,
    pub allow_shell: bool,
    /// While booting: output is collected here so the progress digits
    /// stay quiet.
    pub capture: Option<Vec<u8>>,
    /// Whether `pp`/`hl-type` may write colour escapes (`--color`).
    pub color: bool,
    /// Interactive: reads (and edits) lines from the terminal.
    pub reader: Option<LineReader>,
    /// What was printed since the last newline: the prompt, for redraws.
    pub tail: Vec<u8>,
    /// SQLite for `lib/db/sqlite` (host calls; see sql.rs).
    pub sql: reva_sqlite::Sqlite,
}

impl StdHost {
    /// A host writing to stdout; `reader` for an interactive run.
    pub fn new(allow_shell: bool, color: bool, reader: Option<LineReader>) -> StdHost {
        StdHost {
            out: io::BufWriter::new(io::stdout()),
            fs: StdFs::default(),
            allow_shell,
            capture: None,
            color,
            reader,
            tail: Vec::new(),
            sql: crate::resources::new_sqlite(),
        }
    }

    /// The next line typed (interactive runs only), with its newline.
    pub fn read_line(&mut self) -> Input {
        let _ = self.out.flush();
        let prompt = String::from_utf8_lossy(&self.tail).into_owned();
        self.tail.clear();
        match &mut self.reader {
            Some(r) => r.read_line(&prompt),
            None => Input::Eof,
        }
    }
}

/// A child's wait status, as C's `system` returns it.
#[cfg(unix)]
pub fn wait_status(s: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    s.into_raw()
}

#[cfg(not(unix))]
pub fn wait_status(s: std::process::ExitStatus) -> i32 {
    s.code().unwrap_or(-1)
}
