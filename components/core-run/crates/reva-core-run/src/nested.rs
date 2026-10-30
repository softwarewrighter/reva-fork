//! Nested calls into Forth, and recovering from faults.

use crate::run::Run;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::{RP0, S0};
use reva_core::x86::{Arith, EAX, ESI, ESP};
use reva_core::{Flow, Gate, st};
use reva_core_inspect::words::Dictionary;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Forth called from Rust, and faults.
    pub trait Nested {
        /// Call Forth code at `target` from inside a Rust routine and run it to
        /// completion on the Rust stack.
        ///
        /// Used only for callbacks that are expected to be short and
        /// non-interactive (`do_cr`, `prompt`, `iterate` handlers, overridden
        /// `emit`/`type`).  If such code asks for input that has not arrived,
        /// the host gets one chance to supply it synchronously
        /// ([`Host::read_input_blocking`]); otherwise this faults.  Budgets do
        /// not apply inside a nested call.
        fn call_nested(&mut self, host: &mut dyn Host, target: u32) -> R {
            let saved_eip = self.cpu.eip;
            let entry_esp = self.cpu.r[ESP as usize];
            self.cpu.push32(&mut self.mem, Gate::NestedReturn.addr())?;
            self.cpu.eip = target;
            self.nested += 1;
            let result = loop {
                if let Some(r) = self.nested_flow(host, entry_esp) {
                    break r;
                }
            };
            self.nested -= 1;
            self.cpu.eip = saved_eip;
            result
        }

        /// One step of a nested call: `Some` when the call is over.
        fn nested_flow(&mut self, host: &mut dyn Host, entry_esp: u32) -> Option<R> {
            match self.step(host) {
                Ok(Flow::Continue) => (self.cpu.r[ESP as usize] > entry_esp)
                    .then_some(Err(Fault::UnwoundThroughHost)),
                Ok(Flow::NestedReturn) => Some(Ok(())),
                Ok(Flow::NeedInput) => match host.read_input_blocking() {
                    Some(bytes) if !bytes.is_empty() => {
                        self.input.extend(bytes);
                        None
                    }
                    Some(_) => {
                        self.input_closed = true;
                        None
                    }
                    None => Some(Err(Fault::InputInNestedCall)),
                },
                Ok(Flow::Halt(code)) => {
                    self.halted = Some(code);
                    Some(Ok(()))
                }
                Ok(Flow::Restart) => {
                    self.restart = true;
                    Some(Ok(()))
                }
                Ok(Flow::HostCall) => {
                    self.pending_call = None;
                    Some(Err(Fault::Unavailable(
                        "a host call cannot wait inside a nested call".into(),
                    )))
                }
                Err(f) => Some(Err(f)),
            }
        }

        /// Report a fault and return to the interactive loop, cleaning up the
        /// way `interpret`'s failure path does.
        fn recover(&mut self, host: &mut dyn Host, f: Fault) {
            let eip = self.cpu.eip;
            let at = self.describe_addr(eip);
            host.write(format!("\n*** {f} (in {at}, eip {eip:#010x})\n").as_bytes());
            let _ = self.rollback_definition();
            self.reset_after_fault();
            self.cpu.eip = self.repl_entry();
        }

        /// The interpreter's state and the registers after a fault.
        fn reset_after_fault(&mut self) {
            for (a, v) in [
                (st::IS_COMPILING, 0),
                (st::DIDHEADER, 0),
                (st::SOURCE, 0),
                (st::HANDLER, 0),
                (st::RP0_SAVED, RP0),
            ] {
                let _ = self.mem.write_u32(a, v);
            }
            let tp = self.var(st::TP);
            let _ = self.mem.write_u32(st::TIN, tp);
            self.cpu.r[ESI as usize] = S0;
            self.cpu.r[EAX as usize] = 0;
            self.cpu.r[ESP as usize] = RP0;
            self.cpu.df = false;
        }

        /// Where the interactive loop restarts: reva.f's own
        /// (`exception-reset`, which runs `interp` under `catch`) once
        /// booted; otherwise the core's REPL, or `appstart` if a program
        /// installed its own.
        fn repl_entry(&self) -> u32 {
            let entry = self.var(st::REPL_ENTRY);
            let app = self.var(st::APPSTART);
            if entry != 0 {
                entry
            } else if app == Gate::Hello.addr() {
                Gate::Repl.addr()
            } else {
                app
            }
        }
    }
}
