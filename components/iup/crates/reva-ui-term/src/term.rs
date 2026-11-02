//! The terminal host's state.

use std::collections::VecDeque;

use reva_ui_grid::grid::Grid;
use reva_ui_mirror::mirror::Mirror;

/// The UI as the changes describe it, the screen last drawn (`None`:
/// redraw everything), whether the terminal is in TUI mode, and event
/// lines not yet given (a click is a focus and a click).
#[derive(Default)]
pub struct UiTerm {
    pub mirror: Mirror,
    pub shown: Option<Grid>,
    pub active: bool,
    pub pending: VecDeque<String>,
}
