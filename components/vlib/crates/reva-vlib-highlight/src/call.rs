//! Calls into the library: `hl_color`, `hl_type`.

use crate::render::render;
use crate::table::Table;
use reva_core::Gate;
use reva_core::fault::Fault;
use reva_core::host::Host;

/// At most this many tables in one call.
const MAX_TABLES: u32 = 16;

reva_macros::machine_trait! {
    /// The virtual library `highlight`.
    pub trait Highlight {
        /// A call into the virtual highlight library, or `None` for another gate.
        fn highlight_call(&mut self, host: &mut dyn Host, g: Gate, args: &[u32]) -> Result<Option<u32>, Fault> {
            let a = |i: usize| args.get(i).copied().unwrap_or(0);
            match g {
                Gate::HlColor => Ok(Some(u32::from(host.color()).wrapping_neg())),
                Gate::HlType => self.hl_type(host, (a(0), a(1)), (a(2), a(3))).map(Some),
                _ => Ok(None),
            }
        }

        /// `hl_type` ( src n tables count -- 0 ): the source, coloured with the
        /// `count` tables at `tables` (pairs of address and length), or as
        /// `type` would write it when the host says colour is off.
        fn hl_type(&mut self, host: &mut dyn Host, src: (u32, u32), tables: (u32, u32)) -> Result<u32, Fault> {
            let text = self.mem.slice(src.0, src.1)?.to_vec();
            if !host.color() {
                host.write(&text);
                return Ok(0);
            }
            let mut table = Table::default();
            for t in 0..tables.1.min(MAX_TABLES) {
                let p = tables.0.wrapping_add(t * 8);
                let (ta, tn) = (self.mem.read_u32(p)?, self.mem.read_u32(p + 4)?);
                table.add(&String::from_utf8_lossy(self.mem.slice(ta, tn)?));
            }
            host.write(&render(&text, &table));
            Ok(0)
        }
    }
}
