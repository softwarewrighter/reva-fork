//! The machine as hosts see it: reva-core's `Machine` with reva-words'
//! gateway dispatch installed (rust/DESIGN.md 3.12).  Everything else is
//! reva-core's, reached through `Deref`, so code written against the
//! one-crate core reads the same.

use std::ops::{Deref, DerefMut};

use reva_core::image::ImageError;
pub use reva_core::machine::{DirStream, Dispatch, Next, Status, st};

/// A Reva machine, wired to the gateway routines of reva-words.
pub struct Machine(pub reva_core::Machine);

impl Machine {
    /// A fresh machine; `args` become `argc`/`(argv)` (see
    /// `reva_core::Machine::new`).
    pub fn new(args: &[&str]) -> Machine {
        Machine::wired(reva_core::Machine::new(args))
    }

    /// A machine from a saved image (see `reva_core::Machine::load_image`).
    pub fn load_image(bytes: &[u8], args: &[&str]) -> Result<Machine, ImageError> {
        reva_core::Machine::load_image(bytes, args).map(Machine::wired)
    }

    fn wired(mut m: reva_core::Machine) -> Machine {
        m.dispatch = reva_words::dispatch::dispatch;
        Machine(m)
    }
}

impl Deref for Machine {
    type Target = reva_core::Machine;
    fn deref(&self) -> &reva_core::Machine {
        &self.0
    }
}

impl DerefMut for Machine {
    fn deref_mut(&mut self) -> &mut reva_core::Machine {
        &mut self.0
    }
}
