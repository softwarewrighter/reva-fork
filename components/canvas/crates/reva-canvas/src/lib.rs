//! # reva-canvas — the turtle and its display list
//!
//! [`canvas::Canvas`]: Logo-style turtle graphics on the host side of the
//! VM, drawing into a list of segments; reva-canvas-out turns the list
//! into ops for a window or an SVG file.

pub mod canvas;
pub mod moves;
pub mod pose;
pub mod stack;
