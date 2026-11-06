//! CD's state: canvases (each on an IUP canvas element, with the
//! attributes the next drawing uses and its display list), the active
//! canvas, bitmaps, and the size the renderer gave each IUP canvas.

use crate::draw::{Op, Rgb, encode};
use std::collections::BTreeMap;

/// An IUP element (a canvas), as reva-ui-base numbers them.
pub type Element = u32;

/// CD canvases and bitmaps are numbered from here, apart from IUP's.
pub const FIRST: u32 = 0x4344_0001;

/// The attributes CD draws with, and what has been drawn.
#[derive(Clone, Debug)]
pub struct Canvas {
    pub element: Element,
    pub foreground: Rgb,
    pub background: Rgb,
    pub line_style: i32,
    pub line_width: i32,
    pub interior: i32,
    pub mark_type: i32,
    pub mark_size: i32,
    pub text_size: i32,
    pub ops: Vec<Op>,
    /// `cdBegin`'s mode and the vertices so far.
    pub poly: Option<(i32, Vec<(i32, i32)>)>,
    pub dirty: bool,
}

impl Canvas {
    /// A new canvas on `element`, with CD's defaults (black on white).
    pub fn new(element: Element) -> Canvas {
        Canvas {
            element,
            foreground: 0x000000,
            background: 0xffffff,
            line_style: 0,
            line_width: 1,
            interior: 0,
            mark_type: 1,
            mark_size: 10,
            text_size: 12,
            ops: Vec::new(),
            poly: None,
            dirty: true,
        }
    }
}

/// A bitmap: `w`×`h` pixels, row 0 at the bottom.
#[derive(Clone, Debug)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub pixels: Vec<Rgb>,
}

/// All of a program's CD state.
#[derive(Clone, Debug, Default)]
pub struct Cd {
    pub canvases: BTreeMap<u32, Canvas>,
    pub bitmaps: BTreeMap<u32, Bitmap>,
    pub active: u32,
    pub next: u32,
    /// The size (pixels) the renderer laid each IUP canvas out at.
    pub sizes: BTreeMap<Element, (i32, i32)>,
}

impl Cd {
    /// A new number for a canvas or bitmap.
    pub fn number(&mut self) -> u32 {
        self.next = self.next.max(FIRST) + 1;
        self.next - 1
    }

    /// The dirty canvases' display lists, as `DRAW` values for their IUP
    /// elements; they are clean afterwards.
    pub fn flush(&mut self) -> Vec<(Element, String)> {
        let dirty = self.canvases.values_mut().filter(|c| c.dirty);
        dirty
            .map(|c| {
                c.dirty = false;
                (c.element, encode(&c.ops))
            })
            .collect()
    }

    /// The size of `element`'s canvas (pixels): what the renderer said, or
    /// IUP's default for a canvas.
    pub fn size(&self, element: Element) -> (i32, i32) {
        self.sizes.get(&element).copied().unwrap_or((320, 200))
    }
}
