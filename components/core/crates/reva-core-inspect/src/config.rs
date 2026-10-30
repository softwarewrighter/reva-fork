//! A host's settings for a machine.

use reva_core::translate::Translator;
use reva_core::{Gate, st};

reva_macros::machine_trait! {
    /// How a host sets a machine up.
    pub trait Settings {
        /// Trace every step (gateway or instruction) to stderr.  Slow; for
        /// diagnosing the port, not for production.
        fn set_trace(&mut self, on: bool) {
            self.trace = on;
        }

        /// Install a translator (see reva-translate).
        fn set_translator(&mut self, t: Box<dyn Translator>) {
            self.translator = Some(t);
        }

        /// Turn off the interactive prompt (`reva.f` prints `ok> ` before each
        /// line it reads).  For batch use: piped input, tests, embedding.
        fn quiet_prompt(&mut self) {
            self.mem
                .write_u32(st::PROMPT, Gate::Noop.addr())
                .expect("prompt");
        }
    }
}
