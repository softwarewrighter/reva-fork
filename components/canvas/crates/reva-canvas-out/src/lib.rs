//! # reva-canvas-out — the canvas for windows and files
//!
//! [`ops::CanvasOps`]: the display list as batches of op records for a
//! window (the browser's canvas page, the desktop app's graphics window);
//! [`svg::CanvasSvg`]: the drawing as an SVG document.

pub mod ops;
pub mod svg;
