//! `link` and `-link`: the lists behind `onstartup` and `onexit`.

use reva_core::fault::Fault;
use reva_core::st;
use reva_words_code::emit::Emit;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Linked lists built in code space.
    pub trait Lists: Emit {
        /// `PROC link` ( dataptr listptr -- ): push a node `[prior][dataptr]`
        /// built at `here` onto the front of the list.
        fn link(&mut self) -> R {
            let listptr = self.upop()?;
            let data = self.upop()?;
            let h = self.reserve_code(8)?;
            let prior = self.mem.read_u32(listptr)?;
            self.mem.write_u32(listptr, h)?;
            self.mem.write_u32(h, prior)?;
            self.mem.write_u32(h + 4, data)?;
            self.setv(st::H, h + 8)
        }

        /// `PROC link2` (`-link`): append at the end of the list instead.
        fn link_end(&mut self) -> R {
            let mut p = self.eax();
            loop {
                let next = self.mem.read_u32(p)?;
                if next == 0 {
                    break;
                }
                p = next;
            }
            self.set_eax(p);
            self.link()
        }
    }
}
