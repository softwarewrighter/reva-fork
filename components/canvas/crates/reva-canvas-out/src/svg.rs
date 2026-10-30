//! The drawing as SVG.

use reva_canvas::canvas::{Canvas, Segment, WORLD};
use std::fmt::Write as _;

reva_macros::ext_trait! {
    impl Canvas;
    /// The canvas as an SVG document.
    pub trait CanvasSvg {
        /// The drawing as a standalone SVG document (the window's view: the
        /// `WORLD` square, background included).  Runs of joined segments of
        /// one colour and width become one `<path>`, so files stay small.
        fn svg(&self) -> String {
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
            while i < self.segments.len() {
                i = svg_path(&mut s, &self.segments, i);
            }
            s.push_str("</svg>\n");
            s
        }
    }
}

/// One `<path>` for the run of joined segments of one colour and width
/// starting at `i`; returns the index after the run.
pub fn svg_path(s: &mut String, segs: &[Segment], i: usize) -> usize {
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
        let joined = segs
            .get(j)
            .is_some_and(|n| n.x1 == g.x2 && n.y1 == g.y2 && n.rgb == g.rgb && n.width == g.width);
        if !joined {
            break;
        }
    }
    s.push_str("\"/>\n");
    j
}

/// A coordinate for SVG: two decimals, no trailing zeros, no `-0`.
pub fn num(v: f32) -> String {
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
