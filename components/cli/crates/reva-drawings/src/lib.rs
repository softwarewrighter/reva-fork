//! # reva-drawings — the CLI's turtle drawings as files
//!
//! The CLI has no graphics window, so after each run (each line typed, the
//! files and `-e` texts, piped input) a drawing that changed is written to
//! the next free `turtle-NNN.svg` or `.png` in the current folder, never
//! over an existing file; in a terminal that shows images (iTerm2,
//! WezTerm) a PNG is shown inline too.  `--draw auto|svg|png|off`
//! (docs/turtle.md).
//!
//! [`drawings`]: the mode and the state; [`after`]: what a run left;
//! [`save`]: a new file; [`inline`]: the terminal's inline image.

pub mod after;
pub mod drawings;
pub mod inline;
pub mod save;
