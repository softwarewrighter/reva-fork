//! Lines, boxes, images and polygons on a 2D context, in screen pixels.

use reva_cd::draw::Rgb;
use wasm_bindgen::JsValue;

use web_sys::CanvasRenderingContext2d;

/// `#rrggbb`.
pub(crate) fn css(c: Rgb) -> String {
    format!("#{:06x}", c & 0xff_ffff)
}

/// A line from `(x0, y0)` to `(x1, y1)` in CD's line style and width.
pub(crate) fn line(
    ctx: &CanvasRenderingContext2d,
    (x0, y0, x1, y1): (f64, f64, f64, f64),
    color: &str,
    style: i32,
    width: i32,
) {
    let dash: &[f64] = match style {
        1 => &[6.0, 3.0],
        2 => &[1.0, 2.0],
        3 => &[6.0, 3.0, 1.0, 3.0],
        4 => &[6.0, 3.0, 1.0, 3.0, 1.0, 3.0],
        _ => &[],
    };
    let _ = ctx.set_line_dash(&js_sys_array(dash));
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(f64::from(width.max(1)));
    ctx.begin_path();
    ctx.move_to(x0 + 0.5, y0 + 0.5);
    ctx.line_to(x1 + 0.5, y1 + 0.5);
    ctx.stroke();
    let _ = ctx.set_line_dash(&js_sys_array(&[]));
}

/// A box, filled or its outline: left, top, width, height.
pub(crate) fn rect(
    ctx: &CanvasRenderingContext2d,
    (left, top, w, h): (f64, f64, f64, f64),
    color: &str,
    filled: bool,
) {
    if filled {
        ctx.set_fill_style_str(color);
        ctx.fill_rect(left, top, w, h);
    } else {
        ctx.set_stroke_style_str(color);
        ctx.stroke_rect(left + 0.5, top + 0.5, w - 1.0, h - 1.0);
    }
}

/// An image `w` pixels wide, bottom row first, its lower left at `(x, y)`,
/// each pixel a `cell` (width, height) box.
pub(crate) fn image(
    ctx: &CanvasRenderingContext2d,
    (x, y): (f64, f64),
    (cw, ch): (f64, f64),
    w: i32,
    pixels: &[Rgb],
) {
    for (i, p) in pixels.iter().enumerate() {
        let (col, row) = (f64::from(i as i32 % w), f64::from(i as i32 / w));
        ctx.set_fill_style_str(&css(*p));
        ctx.fill_rect(x + col * cw, y - (row + 1.0) * ch, cw, ch);
    }
}

/// A closed polygon, filled or its outline.
pub(crate) fn poly(
    ctx: &CanvasRenderingContext2d,
    points: &[(f64, f64)],
    color: &str,
    filled: bool,
) {
    ctx.begin_path();
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            ctx.move_to(x, y)
        } else {
            ctx.line_to(x, y)
        }
    }
    ctx.close_path();
    if filled {
        ctx.set_fill_style_str(color);
        ctx.fill();
    } else {
        ctx.set_stroke_style_str(color);
        ctx.stroke();
    }
}

/// A JS array of numbers (a dash pattern).
pub(crate) fn js_sys_array(v: &[f64]) -> JsValue {
    let a = web_sys::js_sys::Array::new();
    for x in v {
        a.push(&JsValue::from_f64(*x));
    }
    a.into()
}
