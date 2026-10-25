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
//! ([`Canvas::take_ops`], once per time slice, never once per line), can
//! ask for all of it again when a window (re)opens ([`Canvas::resend`]),
//! and [`Canvas::svg`] writes it as an SVG file.  Nothing here knows about
//! browsers, windows or files; a host that has no window (the CLI) simply
//! never takes the ops.
//!
//! World coordinates: pixels, origin at the centre, *y up*; heading 0 is
//! north and `right` turns clockwise, as in Logo.  The page shows the
//! square `[-WORLD/2, WORLD/2]²` scaled to its window.
//!
//! ## The op stream
//!
//! [`Canvas::take_ops`] returns records of [`OP_WORDS`] `f32`s, `[op, a,
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

use std::fmt::Write as _;

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

/// Records [`Canvas::take_ops`] returns at most per call (about 512 KiB):
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
    segments: Vec<Segment>,
    stack: Vec<Turtle>,
    /// Segments already handed out by `take_ops`.
    sent: usize,
    /// The next batch starts with CLEAR.
    cleared: bool,
    /// The turtle moved or changed since the last batch.
    moved: bool,
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
fn sin_cos_deg(deg: f64) -> (f64, f64) {
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

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Whether anything was ever drawn or asked for (a host need not open a
    /// window before this).
    pub fn active(&self) -> bool {
        !self.segments.is_empty() || self.cleared || self.moved
    }

    /// Move `dist` pixels along the heading (backwards if negative),
    /// drawing if the pen is down.  Returns false if the drawing is full.
    pub fn forward(&mut self, dist: f64) -> bool {
        let t = self.turtle;
        let (s, c) = sin_cos_deg(t.heading);
        self.move_to(t.x + dist * s, t.y + dist * c)
    }

    /// `forward` by the current step length.
    pub fn step(&mut self) -> bool {
        self.forward(self.turtle.step)
    }

    /// Go to `(x, y)`, drawing if the pen is down.  Returns false if the
    /// drawing is full (the turtle moves anyway).
    pub fn move_to(&mut self, x: f64, y: f64) -> bool {
        let t = self.turtle;
        self.turtle.x = x;
        self.turtle.y = y;
        self.moved = true;
        if !t.pen_down {
            return true;
        }
        if self.segments.len() >= MAX_SEGMENTS {
            return false;
        }
        self.segments.push(Segment {
            x1: t.x as f32,
            y1: t.y as f32,
            x2: x as f32,
            y2: y as f32,
            rgb: t.rgb,
            width: t.width as f32,
        });
        true
    }

    /// Turn clockwise by `deg` degrees (anticlockwise if negative).
    pub fn right(&mut self, deg: f64) {
        self.set_heading(self.turtle.heading + deg);
    }

    pub fn set_heading(&mut self, deg: f64) {
        self.turtle.heading = deg.rem_euclid(360.0);
        self.moved = true;
    }

    /// Back to the centre, facing north (the pen and colour stay).
    pub fn home(&mut self) {
        self.turtle.x = 0.0;
        self.turtle.y = 0.0;
        self.set_heading(0.0);
    }

    /// Erase the drawing (the turtle stays where it is): Logo's `clean`.
    pub fn clean(&mut self) {
        self.segments.clear();
        self.sent = 0;
        self.cleared = true;
        self.moved = true;
        self.full = false;
    }

    /// Erase and start over: drawing, turtle state and stack (Logo's
    /// `clearscreen`).  The step length and visibility are reset too.
    pub fn clear(&mut self) {
        self.clean();
        self.stack.clear();
        self.turtle = Turtle::default();
    }

    /// Multiply the step length by `num/den` (a zero `den` is ignored).
    pub fn scale(&mut self, num: f64, den: f64) {
        if den != 0.0 {
            self.turtle.step *= num / den;
        }
    }

    /// Save the turtle's state; false if the stack is full.
    pub fn push(&mut self) -> bool {
        if self.stack.len() >= MAX_STACK {
            return false;
        }
        self.stack.push(self.turtle);
        true
    }

    /// Restore the last saved state (moving without drawing); false if
    /// nothing was saved.
    pub fn pop(&mut self) -> bool {
        match self.stack.pop() {
            Some(t) => {
                self.turtle = t;
                self.moved = true;
                true
            }
            None => false,
        }
    }

    /// Draw a path given as absolute headings: for each, face it and take
    /// one step.  This is the bulk entry point for array languages (CoSy:
    /// headings are a running sum `+\` of turns).  Returns false if the
    /// drawing filled up.
    pub fn path(&mut self, headings: impl IntoIterator<Item = f64>) -> bool {
        for h in headings {
            self.set_heading(h);
            if !self.step() {
                return false;
            }
        }
        true
    }

    /// Hand out everything drawn since the last call next time (a window
    /// opened, or opened again).
    pub fn resend(&mut self) {
        self.sent = 0;
        self.cleared = true;
        self.moved = true;
    }

    /// Send the turtle's state with the next batch (its visibility changed).
    pub fn resend_turtle(&mut self) {
        self.moved = true;
    }

    /// Whether [`Canvas::take_ops`] has anything to give.
    pub fn has_ops(&self) -> bool {
        self.cleared || self.moved || self.sent < self.segments.len()
    }

    /// The ops since the last call, at most [`MAX_OPS_PER_TAKE`] records
    /// (call again while [`Canvas::has_ops`]); see the module docs.
    pub fn take_ops(&mut self) -> Vec<f32> {
        let mut ops = Vec::new();
        if !self.has_ops() {
            return ops;
        }
        let mut rec = |r: [f32; OP_WORDS]| ops.extend_from_slice(&r);
        if std::mem::take(&mut self.cleared) {
            rec([OP_CLEAR, self.bg as f32, 0., 0., 0., 0., 0., 0.]);
        }
        let end = self.segments.len().min(self.sent + MAX_OPS_PER_TAKE - 2);
        for s in &self.segments[self.sent..end] {
            rec([OP_LINE, s.x1, s.y1, s.x2, s.y2, s.rgb as f32, s.width, 0.]);
        }
        self.sent = end;
        let t = self.turtle;
        rec([
            OP_TURTLE,
            t.x as f32,
            t.y as f32,
            t.heading as f32,
            f32::from(u8::from(self.visible)),
            0.,
            0.,
            0.,
        ]);
        self.moved = false;
        ops
    }

    /// The drawing as a standalone SVG document (the window's view: the
    /// `WORLD` square, background included).  Runs of joined segments of
    /// one colour and width become one `<path>`, so files stay small.
    pub fn svg(&self) -> String {
        let h = WORLD / 2.0;
        let mut s = String::new();
        let _ = writeln!(
            s,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {WORLD} {WORLD}\" \
             width=\"{WORLD}\" height=\"{WORLD}\">",
            -h, -h
        );
        let _ = writeln!(
            s,
            "<rect x=\"{}\" y=\"{}\" width=\"{WORLD}\" height=\"{WORLD}\" fill=\"#{:06x}\"/>",
            -h, -h, self.bg
        );
        let mut i = 0;
        let segs = &self.segments;
        while i < segs.len() {
            let first = segs[i];
            let _ = write!(
                s,
                "<path fill=\"none\" stroke=\"#{:06x}\" stroke-width=\"{}\" \
                 stroke-linecap=\"round\" stroke-linejoin=\"round\" d=\"M{} {}",
                first.rgb,
                num(first.width),
                num(first.x1),
                num(-first.y1)
            );
            let mut j = i;
            loop {
                let g = segs[j];
                let _ = write!(s, " L{} {}", num(g.x2), num(-g.y2));
                j += 1;
                let joined = segs.get(j).is_some_and(|n| {
                    n.x1 == g.x2 && n.y1 == g.y2 && n.rgb == g.rgb && n.width == g.width
                });
                if !joined {
                    break;
                }
            }
            s.push_str("\"/>\n");
            i = j;
        }
        s.push_str("</svg>\n");
        s
    }
}

/// A coordinate for SVG: two decimals, no trailing zeros, no `-0`.
fn num(v: f32) -> String {
    let r = (v as f64 * 100.0).round() / 100.0;
    let r = if r == 0.0 { 0.0 } else { r };
    let mut t = format!("{r:.2}");
    while t.ends_with('0') {
        t.pop();
    }
    if t.ends_with('.') {
        t.pop();
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_closes_exactly() {
        let mut c = Canvas::new();
        for _ in 0..4 {
            c.forward(100.0);
            c.right(90.0);
        }
        assert_eq!(c.segments().len(), 4);
        assert_eq!((c.turtle.x, c.turtle.y, c.turtle.heading), (0.0, 0.0, 0.0));
        assert_eq!(c.segments()[0].y2, 100.0); // north is up
        assert_eq!(c.segments()[1].x2, 100.0); // right turns clockwise
    }

    #[test]
    fn pen_up_moves_without_drawing() {
        let mut c = Canvas::new();
        c.turtle.pen_down = false;
        c.forward(50.0);
        assert!(c.segments().is_empty());
        assert_eq!(c.turtle.y, 50.0);
    }

    #[test]
    fn ops_stream_in_batches_and_resend() {
        let mut c = Canvas::new();
        c.clear();
        c.forward(10.0);
        let ops = c.take_ops();
        assert_eq!(ops.len(), 3 * OP_WORDS); // CLEAR, LINE, TURTLE
        assert_eq!(ops[0], OP_CLEAR);
        assert_eq!(ops[OP_WORDS], OP_LINE);
        assert_eq!(ops[2 * OP_WORDS], OP_TURTLE);
        assert!(!c.has_ops());
        assert!(c.take_ops().is_empty());
        c.resend();
        assert_eq!(c.take_ops().len(), 3 * OP_WORDS);
    }

    #[test]
    fn batches_are_capped() {
        let mut c = Canvas::new();
        c.path((0..40_000).map(|i| f64::from(i % 4) * 90.0));
        let mut lines = 0;
        let mut takes = 0;
        while c.has_ops() {
            let ops = c.take_ops();
            assert!(ops.len() <= MAX_OPS_PER_TAKE * OP_WORDS);
            lines += ops.chunks(OP_WORDS).filter(|r| r[0] == OP_LINE).count();
            takes += 1;
        }
        assert_eq!(lines, 40_000);
        assert_eq!(takes, 3);
    }

    #[test]
    fn the_drawing_has_a_limit() {
        let mut c = Canvas::new();
        c.turtle.step = 0.0;
        assert!(c.path(std::iter::repeat_n(0.0, MAX_SEGMENTS)));
        assert!(!c.step());
        assert_eq!(c.segments().len(), MAX_SEGMENTS);
    }

    #[test]
    fn push_and_pop_restore_the_turtle() {
        let mut c = Canvas::new();
        c.push();
        c.forward(30.0);
        c.right(45.0);
        assert!(c.pop());
        assert_eq!(c.turtle, Turtle::default());
        assert!(!c.pop());
    }

    #[test]
    fn svg_joins_runs_into_paths() {
        let mut c = Canvas::new();
        c.forward(100.0);
        c.right(90.0);
        c.forward(100.0);
        c.turtle.rgb = 0xff0000;
        c.forward(-50.0);
        let svg = c.svg();
        assert_eq!(svg.matches("<path").count(), 2);
        assert!(svg.contains("d=\"M0 0 L0 -100 L100 -100\""), "{svg}");
        assert!(svg.contains("stroke=\"#ff0000\""));
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(1.5), "1.5");
    }
}
