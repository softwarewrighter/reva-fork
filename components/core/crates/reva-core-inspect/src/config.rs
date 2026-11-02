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

        /// Remember reva.f's own prompt: call right after booting reva.f
        /// (not from an image), before any program runs.  Kept in images.
        fn record_prompt(&mut self) {
            let p = self.mem.read_u32(st::PROMPT).expect("prompt");
            self.mem.write_u32(st::PROMPT_DEFAULT, p).expect("prompt");
        }

        /// Turn off the interactive prompt (`reva.f` prints `ok> ` before each
        /// line it reads).  For batch use: piped input, tests, embedding.
        /// Only reva.f's own prompt: one a program installed (an image's)
        /// is kept.  Without the record (older images) it is turned off.
        fn quiet_prompt(&mut self) {
            let p = self.mem.read_u32(st::PROMPT).expect("prompt");
            let own = self.mem.read_u32(st::PROMPT_DEFAULT).expect("prompt");
            if own == 0 || p == own {
                self.mem
                    .write_u32(st::PROMPT, Gate::Noop.addr())
                    .expect("prompt");
            }
        }

        /// reva.f's prompt back, if an image was saved with it turned off
        /// (`reva --save-image` runs without a prompt) and starts at a
        /// terminal.
        fn restore_prompt(&mut self) {
            let p = self.mem.read_u32(st::PROMPT).expect("prompt");
            let own = self.mem.read_u32(st::PROMPT_DEFAULT).expect("prompt");
            if p == Gate::Noop.addr() && own != 0 {
                self.mem.write_u32(st::PROMPT, own).expect("prompt");
            }
        }
    }
}
