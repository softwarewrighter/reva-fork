//! # The virtual library `turtle`
//!
//! Forth reaches the [`Canvas`](crate::canvas::Canvas) the way it reaches libc: `" turtle" lib`
//! then `func:` per function, resolved by `(lib)`/`(func)` and called with
//! `(call)` (see `ffi.rs` and `lib/turtle`).  Arguments are integers
//! (pixels, degrees, `0xRRGGBB`); the turtle's own arithmetic is `f64`.
//!
//! Every function returns a value (C `int`); those that only act return 0,
//! so `vfunc:` drops it.

use crate::fault::Fault;
use crate::gates::Gate;
use crate::host::Host;
use crate::vfs::Access;
use reva_canvas_out::ops::CanvasOps;
use reva_canvas_out::svg::CanvasSvg;

type R<T = ()> = Result<T, Fault>;

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

fn int(v: u32) -> f64 {
    f64::from(v as i32)
}

reva_macros::machine_trait! {
    /// The virtual library `turtle`.
    pub trait Turtle {
        /// Note once per drawing that it is full.
        fn tt_drew(&mut self, host: &mut dyn Host, ok: bool) {
            if !ok && !self.canvas.full {
                self.canvas.full = true;
                host.diag(&format!(
                    "note: turtle drawing limit ({} segments) reached; `clear` starts over\n",
                    crate::canvas::MAX_SEGMENTS
                ));
            }
    }

    /// Call turtle function `g` with `args` (C order).  `None`: not one.
    fn turtle_call(&mut self, host: &mut dyn Host, g: Gate, args: &[u32]) -> R<Option<u32>> {
        let a = |i: usize| args.get(i).copied().unwrap_or(0);
        let c = &mut self.canvas;
        let r = match g {
            Gate::TtForward => {
                let ok = c.forward(int(a(0)));
                self.tt_drew(host, ok);
                0
            }
            Gate::TtStep => {
                let ok = c.step();
                self.tt_drew(host, ok);
                0
            }
            Gate::TtRight => {
                c.right(int(a(0)));
                0
            }
            Gate::TtPen => {
                c.turtle.pen_down = a(0) != 0;
                0
            }
            Gate::TtColor => {
                c.turtle.rgb = a(0) & 0xff_ffff;
                0
            }
            Gate::TtWidth => {
                c.turtle.width = int(a(0)).clamp(0.0, 100.0);
                0
            }
            Gate::TtHome => {
                c.home();
                0
            }
            Gate::TtClear => {
                c.clear();
                0
            }
            Gate::TtClean => {
                c.clean();
                0
            }
            Gate::TtSetxy => {
                let ok = c.move_to(int(a(0)), int(a(1)));
                self.tt_drew(host, ok);
                0
            }
            Gate::TtSetheading => {
                c.set_heading(int(a(0)));
                0
            }
            Gate::TtSetstep => {
                c.turtle.step = int(a(0));
                0
            }
            Gate::TtScale => {
                c.scale(int(a(0)), int(a(1)));
                0
            }
            Gate::TtShow => {
                c.visible = a(0) != 0;
                c.resend_turtle();
                0
            }
            Gate::TtX => c.turtle.x.round() as i32 as u32,
            Gate::TtY => c.turtle.y.round() as i32 as u32,
            Gate::TtHeading => c.turtle.heading.round() as i32 as u32,
            Gate::TtPush => u32::from(c.push()).wrapping_neg(),
            Gate::TtPop => u32::from(c.pop()).wrapping_neg(),
            Gate::TtPath => {
                // ( a n ): n cells of headings at a
                let (p, n) = (a(0), a(1));
                let bytes = self.mem.slice(p, n.saturating_mul(4))?;
                let hs: Vec<f64> = bytes
                    .chunks_exact(4)
                    .map(|b| f64::from(i32::from_le_bytes([b[0], b[1], b[2], b[3]])))
                    .collect();
                let ok = self.canvas.path(hs);
                self.tt_drew(host, ok);
                0
            }
            Gate::TtSvg => {
                // ( a n ): file name; returns 0 or -errno, like the file words
                let name = String::from_utf8_lossy(self.mem.slice(a(0), a(1))?).into_owned();
                let svg = self.canvas.svg();
                write_file(host, &name, svg.as_bytes()).map_or_else(|e| e.wrapping_neg(), |()| 0)
            }
            Gate::TtBg => {
                c.bg = a(0) & 0xff_ffff;
                c.resend();
                0
            }
            Gate::TtSegments => c.segments().len() as u32,
            _ => return Ok(None),
        };
        Ok(Some(r))
    }
    }
}

fn write_file(host: &mut dyn Host, path: &str, data: &[u8]) -> Result<(), u32> {
    const ENOSYS: u32 = 38;
    let fs = host.fs().ok_or(ENOSYS)?;
    let fd = fs.open(path, Access::Create)?;
    let r = fs.write(fd, data).map(drop);
    let _ = fs.close(fd);
    r
}
