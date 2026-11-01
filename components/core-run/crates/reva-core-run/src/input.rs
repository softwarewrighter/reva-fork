//! Input from the host, and stopping running code.

use crate::run::Run;
use reva_core::Status;
use reva_core::host::Host;

reva_macros::machine_trait! {
    /// The machine's input queue.
    pub trait Input {
        /// Queue input bytes (keyboard data, pasted text, piped stdin).
        fn feed(&mut self, bytes: &[u8]) {
            self.input.extend(bytes);
        }

        /// Take the input not yet read, and whether input was closed, for the
        /// machine that replaces this one after `Status::Restart`.  The rest
        /// of the line being interpreted is already in the VM and is dropped.
        fn take_input(&mut self) -> (Vec<u8>, bool) {
            (self.input.drain(..).collect(), self.input_closed)
        }

        /// Mark the end of input: once the queue drains, `key` returns EOF and
        /// the interpreter exits as Reva did on Ctrl-D.
        fn close_input(&mut self) {
            self.input_closed = true;
        }

        /// Stop the running code (Ctrl-C, a Stop button): at its next step,
        /// `run` drops queued input and recovers as from a fault
        /// (`*** interrupted (in WORD ...)`), back to the interpreter loop with
        /// the dictionary kept and a half-made definition removed.
        ///
        /// The host calls this between `run` calls, so code inside a nested
        /// call (which runs to completion within one step) cannot be stopped
        /// this way; a host that must always regain control runs the VM where
        /// it can discard it (the browser's worker: `terminate()`).
        fn interrupt(&mut self) {
            self.interrupt = true;
        }

        /// Run until the machine needs input or halts.
        fn run_until_blocked(&mut self, host: &mut dyn Host) -> Status {
            loop {
                match self.run(host, 1 << 20) {
                    Status::Yielded => continue,
                    s => return s,
                }
            }
        }
    }
}
