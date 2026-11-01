//! # reva-term — reading a line at a terminal
//!
//! Reading a line at a terminal with reva-lineedit: raw
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

pub mod reader;
