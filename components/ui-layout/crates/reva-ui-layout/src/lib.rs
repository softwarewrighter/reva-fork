//! # reva-ui-layout — IUP's layout, in character cells
//!
//! Where every element of a dialog goes, computed as IUP does (natural
//! sizes, SIZE, EXPAND, boxes with MARGIN, GAP and ALIGNMENT) but in the
//! cells of a text grid, the unit of the TUI every host shows
//! (docs/ui-model.md): rectangles and sizes ([`geom`]), sizes from
//! attributes ([`units`]), natural sizes and expansion ([`natural`]),
//! placing ([`place`]), boxes ([`boxes`]).  Pure: a tree in, rectangles out.

pub mod boxes;
pub mod geom;
pub mod natural;
pub mod place;
pub mod units;
