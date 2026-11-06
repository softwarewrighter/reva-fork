//! # reva-cd-list — the display list of CD canvases
//!
//! [`draw`]: the operations a CD canvas records and their text form, the
//! value of the IUP canvas's `DRAW` attribute that renderers paint;
//! [`codec`]: one operation's text, and reading it back.

mod codec;
pub mod draw;
