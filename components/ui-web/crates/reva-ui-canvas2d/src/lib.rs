//! # reva-ui-canvas2d — an IUP canvas's drawing, on an HTML canvas
//!
//! The page's TUI lays an HTML `<canvas>` over each IUP canvas element and
//! paints its `DRAW` attribute (reva-cd's display list) with [`paint`]:
//! CD's coordinates (origin at the bottom left) flipped to the screen's,
//! its line styles as dash patterns, its marks, boxes, texts and bitmaps.

mod marks;
pub mod paint;
mod shapes;
