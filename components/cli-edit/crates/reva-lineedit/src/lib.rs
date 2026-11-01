//! # reva-lineedit — the terminal line editor
//!
//! The terminal line editor's logic: key decoding, editing, history
//! recall, Ctrl-R search and drawing.  No I/O here (see reva-term), so all
//! of it is unit tested.
//!
//! The keys are the web page's (`www/lineedit.js`); `docs/repl.md` has
//! the table.  The line is drawn on one terminal row after the prompt,
//! scrolling sideways when it is wider than the terminal, so a redraw is
//! always "carriage return, prompt, visible text, clear to end of row,
//! move to the cursor" and never depends on where wrapped rows went.

pub mod editor;
pub mod keymap;
pub mod motion;
pub mod view;
