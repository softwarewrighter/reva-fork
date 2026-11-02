//! The drawing as PNG.

use reva_canvas::canvas::{Canvas, Segment, WORLD};
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

reva_macros::ext_trait! {
    impl Canvas;
    /// The canvas as a PNG image.
    pub trait CanvasPng {
        /// The drawing as a PNG file's bytes: the `WORLD` square, one pixel
        /// per unit, background included (as the SVG); `None` if encoding
        /// failed.
        fn png(&self) -> Option<Vec<u8>> {
            let side = WORLD as u32;
            let mut pm = Pixmap::new(side, side)?;
            pm.fill(color(self.bg));
            let h = (WORLD / 2.0) as f32;
            let at = Transform::from_row(1.0, 0.0, 0.0, -1.0, h, h);
            for g in &self.segments {
                stroke(&mut pm, g, at);
            }
            pm.encode_png().ok()
        }
    }
}

/// One segment, in its colour and width, with round ends (as the SVG).
pub fn stroke(pm: &mut Pixmap, g: &Segment, at: Transform) {
    let mut pb = PathBuilder::new();
    pb.move_to(g.x1, g.y1);
    pb.line_to(g.x2, g.y2);
    let Some(path) = pb.finish() else { return };
    let mut paint = Paint::default();
    paint.set_color(color(g.rgb));
    paint.anti_alias = true;
    let st = Stroke {
        width: g.width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    pm.stroke_path(&path, &paint, &st, at, None);
}

/// A 0xRRGGBB colour, opaque.
pub fn color(rgb: u32) -> Color {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color::from_rgba8(r, g, b, 255)
}
