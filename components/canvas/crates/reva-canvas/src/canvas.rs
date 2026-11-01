//! # The canvas: a turtle and the display list it draws
//!
//! Logo-style turtle graphics, kept entirely on the host side of the VM:
//! the turtle's position and heading are `f64`, and the trigonometry is
//! Rust's, so Forth code needs no floating point (the portable core has no
//! x87 yet) and integer arguments lose nothing to rounding as a drawing
//! recurses.  Forth reaches it through the virtual library `turtle` (see
//! `ffi.rs` and `lib/turtle`).
//!
//! Everything drawn goes into a **display list** of segments.  The list is
//! the drawing: a host streams the part it has not seen yet to a window
//! (reva-canvas-out's `take_ops`, once per time slice, never once per
//! line), can ask for all of it again when a window (re)opens (`resend`),
//! and `svg` writes it as an SVG file.  The turtle's moves are in
//! [`crate::moves`], [`crate::pose`] and [`crate::stack`].  Nothing here knows about
//! browsers, windows or files; a host that has no window (the CLI) simply
//! never takes the ops.
//!
//! World coordinates: pixels, origin at the centre, *y up*; heading 0 is
//! north and `right` turns clockwise, as in Logo.  The page shows the
//! square `[-WORLD/2, WORLD/2]²` scaled to its window.
//!
//! ## The op stream
//!
//! `take_ops` returns records of [`OP_WORDS`] `f32`s, `[op, a,
//! b, c, d, e, f, g]`; integers (colours, flags) are exact in an `f32`.
//!
//! | op | name    | a, b, c, d, e, f, g |
//! |----|---------|---------------------|
//! | 1  | CLEAR   | background `0xRRGGBB` |
//! | 2  | LINE    | x1, y1, x2, y2, `0xRRGGBB`, width |
//! | 3  | TURTLE  | x, y, heading (degrees), visible (0/1) |
//!
//! A batch that starts a drawing over begins with CLEAR; the last record of
//! every non-empty batch is TURTLE, where the turtle is now.

/// `f32`s per op record.
pub const OP_WORDS: usize = 8;
pub const OP_CLEAR: f32 = 1.0;
pub const OP_LINE: f32 = 2.0;
pub const OP_TURTLE: f32 = 3.0;

/// Width and height of the square the page shows, in world pixels.
pub const WORLD: f64 = 800.0;

/// Segments kept per drawing (about 24 bytes each).  Past this the turtle
/// still moves but draws nothing, and the VM notes it once, so a runaway
/// recursion cannot exhaust memory or bury a window in lines.
pub const MAX_SEGMENTS: usize = 1_000_000;

/// Records `take_ops` returns at most per call (about 512 KiB):
/// one message to a window stays small however fast the turtle draws.
pub const MAX_OPS_PER_TAKE: usize = 16_384;

/// Depth of the turtle-state stack (`push`/`pop`, L-system brackets).
pub const MAX_STACK: usize = 4096;

pub const DEFAULT_COLOR: u32 = 0x1f6f5c;
pub const DEFAULT_BG: u32 = 0xfbfaf7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub rgb: u32,
    pub width: f32,
}

/// Where the turtle is and how it draws: what `push` saves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turtle {
    pub x: f64,
    pub y: f64,
    /// Degrees clockwise from north, in `[0, 360)`.
    pub heading: f64,
    pub pen_down: bool,
    pub rgb: u32,
    pub width: f64,
    /// The length `step` moves; `scale` multiplies it.
    pub step: f64,
}

impl Default for Turtle {
    fn default() -> Turtle {
        Turtle {
            x: 0.0,
            y: 0.0,
            heading: 0.0,
            pen_down: true,
            rgb: DEFAULT_COLOR,
            width: 1.0,
            step: 10.0,
        }
    }
}

#[derive(Debug)]
pub struct Canvas {
    pub turtle: Turtle,
    pub visible: bool,
    pub bg: u32,
    pub segments: Vec<Segment>,
    pub stack: Vec<Turtle>,
    /// Segments already handed out by `take_ops`.
    pub sent: usize,
    /// The next batch starts with CLEAR.
    pub cleared: bool,
    /// The turtle moved or changed since the last batch.
    pub moved: bool,
    /// `MAX_SEGMENTS` was reached (reported once per drawing).
    pub full: bool,
}

impl Default for Canvas {
    fn default() -> Canvas {
        Canvas::new()
    }
}

/// `sin` and `cos` of `deg` degrees, exact at multiples of 90 so that
/// right-angle curves (Hilbert, dragon) land on whole pixels.
pub fn sin_cos_deg(deg: f64) -> (f64, f64) {
    let d = deg.rem_euclid(360.0);
    if d == 0.0 {
        (0.0, 1.0)
    } else if d == 90.0 {
        (1.0, 0.0)
    } else if d == 180.0 {
        (0.0, -1.0)
    } else if d == 270.0 {
        (-1.0, 0.0)
    } else {
        d.to_radians().sin_cos()
    }
}

impl Canvas {
    pub fn new() -> Canvas {
        Canvas {
            turtle: Turtle::default(),
            visible: true,
            bg: DEFAULT_BG,
            segments: Vec::new(),
            stack: Vec::new(),
            sent: 0,
            cleared: false,
            moved: false,
            full: false,
        }
    }
}
