//! CD's marks on a 2D context.

use web_sys::CanvasRenderingContext2d;

/// A CD mark of `kind` and `size` (pixels) centred at `x`, `y`: plus, star,
/// circle, x, box, diamond, then the hollow circle, box and diamond.
pub(crate) fn mark(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    kind: i32,
    size: f64,
    color: &str,
) {
    let r = (size / 2.0).max(1.0);
    ctx.set_fill_style_str(color);
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(1.0);
    ctx.begin_path();
    match kind {
        0 | 1 | 3 => {
            if kind != 3 {
                ctx.move_to(x - r, y);
                ctx.line_to(x + r, y);
                ctx.move_to(x, y - r);
                ctx.line_to(x, y + r);
            }
            if kind != 0 {
                ctx.move_to(x - r, y - r);
                ctx.line_to(x + r, y + r);
                ctx.move_to(x - r, y + r);
                ctx.line_to(x + r, y - r);
            }
            ctx.stroke();
        }
        2 | 6 => {
            let _ = ctx.arc(x, y, r, 0.0, std::f64::consts::TAU);
            if kind == 2 { ctx.fill() } else { ctx.stroke() }
        }
        4 | 7 => {
            ctx.rect(x - r, y - r, 2.0 * r, 2.0 * r);
            if kind == 4 { ctx.fill() } else { ctx.stroke() }
        }
        _ => {
            ctx.move_to(x, y - r);
            ctx.line_to(x + r, y);
            ctx.line_to(x, y + r);
            ctx.line_to(x - r, y);
            ctx.close_path();
            if kind == 5 { ctx.fill() } else { ctx.stroke() }
        }
    }
}
