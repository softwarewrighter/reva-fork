//! # reva-canvas-png — the canvas as a PNG image
//!
//! [`png::CanvasPng`]: the drawing rasterised with tiny-skia (BSD-3-Clause),
//! the same view as the SVG and the window.  A crate of its own so the
//! rasteriser stays out of the wasm build.

pub mod png;
