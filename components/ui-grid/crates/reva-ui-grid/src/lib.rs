//! # reva-ui-grid — the TUI's grid of cells
//!
//! A screen of character cells, each with a colour role, and the text
//! cursor ([`grid`]); drawing characters, text and boxes into it, and the
//! grid as text for goldens ([`cells`]).  reva-ui-paint paints dialogs
//! into it; backends draw the cells (the web page, the terminal), mapping
//! the roles to their theme's colours (docs/ui-model.md).

pub mod cells;
pub mod grid;
