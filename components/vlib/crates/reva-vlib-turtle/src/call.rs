//! Calls into the library: the canvas's functions, paths, SVG files.

use crate::state::{draw, query, set};
use reva_canvas_out::svg::CanvasSvg;
use reva_core::Gate;
use reva_core::canvas::MAX_SEGMENTS;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::Access;

type R<T = ()> = Result<T, Fault>;

/// Write `data` to the host file `path`; `Err(errno)` on failure.
pub fn write_file(host: &mut dyn Host, path: &str, data: &[u8]) -> Result<(), u32> {
    const ENOSYS: u32 = 38;
    let fs = host.fs().ok_or(ENOSYS)?;
    let fd = fs.open(path, Access::Create)?;
    let r = fs.write(fd, data).map(drop);
    let _ = fs.close(fd);
    r
}

reva_macros::machine_trait! {
    /// The virtual library `turtle`.
    pub trait Turtle {
        /// Note once per drawing that it is full.
        fn tt_drew(&mut self, host: &mut dyn Host, ok: bool) {
            if !ok && !self.canvas.full {
                self.canvas.full = true;
                host.diag(&format!(
                    "note: turtle drawing limit ({MAX_SEGMENTS} segments) reached; `clear` starts over\n"
                ));
            }
        }

        /// Call turtle function `g` with `args` (C order).  `None`: not one.
        fn turtle_call(&mut self, host: &mut dyn Host, g: Gate, args: &[u32]) -> R<Option<u32>> {
            if let Some(ok) = draw(&mut self.canvas, g, args) {
                self.tt_drew(host, ok);
                return Ok(Some(0));
            }
            if let Some(r) = set(&mut self.canvas, g, args).or_else(|| query(&mut self.canvas, g)) {
                return Ok(Some(r));
            }
            let a = |i: usize| args.get(i).copied().unwrap_or(0);
            match g {
                Gate::TtPath => self.tt_path(host, a(0), a(1)).map(Some),
                Gate::TtSvg => self.tt_svg(host, a(0), a(1)).map(Some),
                _ => Ok(None),
            }
        }

        /// `tt_path` ( a n ): n cells of headings at a.
        fn tt_path(&mut self, host: &mut dyn Host, p: u32, n: u32) -> R<u32> {
            let bytes = self.mem.slice(p, n.saturating_mul(4))?;
            let hs: Vec<f64> = bytes
                .chunks_exact(4)
                .map(|b| f64::from(i32::from_le_bytes([b[0], b[1], b[2], b[3]])))
                .collect();
            let ok = self.canvas.path(hs);
            self.tt_drew(host, ok);
            Ok(0)
        }

        /// `tt_svg` ( a n ): the drawing to the file named; 0 or -errno, like
        /// the file words.
        fn tt_svg(&mut self, host: &mut dyn Host, a: u32, n: u32) -> R<u32> {
            let name = String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned();
            let svg = self.canvas.svg();
            Ok(write_file(host, &name, svg.as_bytes()).map_or_else(|e| e.wrapping_neg(), |()| 0))
        }
    }
}
