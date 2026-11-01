//! # reva-ui-paint — the TUI, one look everywhere
//!
//! The model's shown dialogs (a reva-ui-mirror copy) painted into
//! reva-ui-grid's cells (docs/ui-model.md): each element as the TUI draws
//! it ([`draw`]), text and lists in their panes ([`fields`]), and whole
//! screens with what is under a click ([`paint`]).

pub mod draw;
pub mod fields;
pub mod paint;
