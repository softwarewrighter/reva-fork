//! Painting a display list on a 2D context.

use reva_cd::draw::{Op, decode};

use crate::marks::mark;
use crate::shapes::{css, image, line, poly, rect};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Paint `draw` (a `DRAW` value) on `canvas`, sized to its box in CSS
/// pixels first; that size, for the canvas's `size` event.
pub fn paint(canvas: &HtmlCanvasElement, draw: &str) -> (i32, i32) {
    let (w, h) = (canvas.client_width().max(1), canvas.client_height().max(1));
    if canvas.width() != w as u32 || canvas.height() != h as u32 {
        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
    }
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok());
    if let Some(ctx) = ctx {
        for op in decode(draw) {
            one(&ctx, &op, h);
        }
    }
    (w, h)
}

/// One operation; `h` flips CD's y (up) to the screen's (down).
fn one(ctx: &CanvasRenderingContext2d, op: &Op, h: i32) {
    let y = |v: i32| f64::from(h - 1 - v);
    match op {
        Op::Clear(c) => {
            ctx.set_fill_style_str(&css(*c));
            ctx.fill_rect(0.0, 0.0, 1e6, 1e6);
        }
        Op::Line {
            from,
            to,
            color,
            style,
            width,
        } => {
            let ends = (f64::from(from.0), y(from.1), f64::from(to.0), y(to.1));
            line(ctx, ends, &css(*color), *style, *width);
        }
        Op::Box {
            lo,
            hi,
            color,
            filled,
        } => {
            let (x0, x1) = (lo.0.min(hi.0), lo.0.max(hi.0));
            let (y0, y1) = (lo.1.min(hi.1), lo.1.max(hi.1));
            let r = (
                f64::from(x0),
                y(y1),
                f64::from(x1 - x0 + 1),
                f64::from(y1 - y0 + 1),
            );
            rect(ctx, r, &css(*color), *filled);
        }
        Op::Mark {
            at,
            kind,
            size,
            color,
        } => mark(
            ctx,
            f64::from(at.0),
            y(at.1),
            *kind,
            f64::from(*size),
            &css(*color),
        ),
        _ => more(ctx, op, y),
    }
}

/// Texts, images and polygons; `y` flips a y as [`one`] does.
fn more(ctx: &CanvasRenderingContext2d, op: &Op, y: impl Fn(i32) -> f64) {
    match op {
        Op::Text {
            at,
            color,
            size,
            text,
        } => {
            ctx.set_fill_style_str(&css(*color));
            ctx.set_font(&format!("{}px monospace", (*size).max(6)));
            let _ = ctx.fill_text(text, f64::from(at.0), y(at.1));
        }
        Op::Image {
            at,
            size,
            w,
            h: ih,
            pixels,
        } => {
            let cell = (
                f64::from(size.0) / f64::from((*w).max(1)),
                f64::from(size.1) / f64::from((*ih).max(1)),
            );
            image(
                ctx,
                (f64::from(at.0), y(at.1) + 1.0),
                cell,
                (*w).max(1),
                pixels,
            );
        }
        Op::Poly {
            points,
            color,
            filled,
        } => {
            let pts: Vec<(f64, f64)> = points
                .iter()
                .map(|p| (f64::from(p.0) + 0.5, y(p.1) + 0.5))
                .collect();
            poly(ctx, &pts, &css(*color), *filled);
        }
        _ => {}
    }
}
