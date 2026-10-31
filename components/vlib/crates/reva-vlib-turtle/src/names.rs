//! The library's names: `(lib)`'s handle, its functions and gateways.

use reva_core::Gate;

/// Handle `(lib)` returns for the virtual turtle library ("TRTL").
pub const TURTLE_HANDLE: u32 = 0x5452_544c;

/// Library names accepted for it.
pub const TURTLE_NAMES: &[&str] = &["turtle", "libturtle", "libturtle.so", "turtle.dll"];

/// The functions, their gateways, and how many arguments each takes.
pub const TURTLE_FUNCS: &[(&str, Gate, u32)] = &[
    ("tt_forward", Gate::TtForward, 1),
    ("tt_step", Gate::TtStep, 0),
    ("tt_right", Gate::TtRight, 1),
    ("tt_pen", Gate::TtPen, 1),
    ("tt_color", Gate::TtColor, 1),
    ("tt_width", Gate::TtWidth, 1),
    ("tt_home", Gate::TtHome, 0),
    ("tt_clear", Gate::TtClear, 0),
    ("tt_clean", Gate::TtClean, 0),
    ("tt_setxy", Gate::TtSetxy, 2),
    ("tt_setheading", Gate::TtSetheading, 1),
    ("tt_setstep", Gate::TtSetstep, 1),
    ("tt_scale", Gate::TtScale, 2),
    ("tt_show", Gate::TtShow, 1),
    ("tt_x", Gate::TtX, 0),
    ("tt_y", Gate::TtY, 0),
    ("tt_heading", Gate::TtHeading, 0),
    ("tt_push", Gate::TtPush, 0),
    ("tt_pop", Gate::TtPop, 0),
    ("tt_path", Gate::TtPath, 2),
    ("tt_svg", Gate::TtSvg, 2),
    ("tt_bg", Gate::TtBg, 1),
    ("tt_segments", Gate::TtSegments, 0),
];

/// The gateway of turtle function `name`.
pub fn lookup(name: &str) -> Option<Gate> {
    TURTLE_FUNCS
        .iter()
        .find(|(f, _, _)| *f == name)
        .map(|&(_, g, _)| g)
}
