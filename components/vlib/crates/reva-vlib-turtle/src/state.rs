//! The turtle functions that only touch the canvas, as pure functions of
//! it: moves (which may fill the drawing), settings, and queries.

use reva_canvas_out::ops::CanvasOps;
use reva_core::Gate;
use reva_core::canvas::Canvas;

/// Argument `i` of a call (0 if missing) as a signed number.
pub fn int(args: &[u32], i: usize) -> f64 {
    f64::from(args.get(i).copied().unwrap_or(0) as i32)
}

/// A move: whether the drawing still had room, or `None` for another gate.
pub fn draw(c: &mut Canvas, g: Gate, args: &[u32]) -> Option<bool> {
    Some(match g {
        Gate::TtForward => c.forward(int(args, 0)),
        Gate::TtStep => c.step(),
        Gate::TtSetxy => c.move_to(int(args, 0), int(args, 1)),
        _ => return None,
    })
}

/// A setting (returns 0), or `None` for another gate.
pub fn set(c: &mut Canvas, g: Gate, args: &[u32]) -> Option<u32> {
    let a = |i: usize| args.get(i).copied().unwrap_or(0);
    match g {
        Gate::TtRight => c.right(int(args, 0)),
        Gate::TtPen => c.turtle.pen_down = a(0) != 0,
        Gate::TtColor => c.turtle.rgb = a(0) & 0xff_ffff,
        Gate::TtWidth => c.turtle.width = int(args, 0).clamp(0.0, 100.0),
        Gate::TtHome => c.home(),
        Gate::TtClear => c.clear(),
        Gate::TtClean => c.clean(),
        Gate::TtSetheading => c.set_heading(int(args, 0)),
        Gate::TtSetstep => c.turtle.step = int(args, 0),
        Gate::TtScale => c.scale(int(args, 0), int(args, 1)),
        Gate::TtShow => {
            c.visible = a(0) != 0;
            c.resend_turtle();
        }
        Gate::TtBg => {
            c.bg = a(0) & 0xff_ffff;
            c.resend();
        }
        _ => return None,
    }
    Some(0)
}

/// A query (position, heading, stack, segment count), or `None`.
pub fn query(c: &mut Canvas, g: Gate) -> Option<u32> {
    Some(match g {
        Gate::TtX => c.turtle.x.round() as i32 as u32,
        Gate::TtY => c.turtle.y.round() as i32 as u32,
        Gate::TtHeading => c.turtle.heading.round() as i32 as u32,
        Gate::TtPush => u32::from(c.push()).wrapping_neg(),
        Gate::TtPop => u32::from(c.pop()).wrapping_neg(),
        Gate::TtSegments => c.segments().len() as u32,
        _ => return None,
    })
}
