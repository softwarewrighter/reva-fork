//! What a host reads of a machine.

use reva_core::canvas::Canvas;
use reva_core::layout::S0;
use reva_core::mem::Memory;
use reva_core::x86::{Cpu, EAX, ESI};

reva_macros::machine_trait! {
    /// The machine's state, for hosts.
    pub trait View {
        fn cpu(&self) -> &Cpu {
            &self.cpu
        }

        fn memory(&self) -> &Memory {
            &self.mem
        }

        /// The data stack, bottom first, TOS last.  Reva keeps TOS in EAX and
        /// the cell at `S0-4` is a placeholder, so depth is `(S0-ESI)/4`.
        fn data_stack(&self) -> Vec<u32> {
            let esi = self.cpu.r[ESI as usize];
            if esi >= S0 {
                return Vec::new();
            }
            let depth = (S0 - esi) / 4;
            let mut v: Vec<u32> = (1..depth)
                .map(|k| self.mem.read_u32(S0 - 4 - 4 * k).unwrap_or(0))
                .collect();
            v.push(self.cpu.r[EAX as usize]);
            v
        }

        /// The turtle and its drawing (reva-vlib's `turtle`), for a host to
        /// stream or save.
        fn canvas(&mut self) -> &mut Canvas {
            &mut self.canvas
        }

        /// The exit code, once halted.
        fn halted(&self) -> Option<i32> {
            self.halted
        }
    }
}
