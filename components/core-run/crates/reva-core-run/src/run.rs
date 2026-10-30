//! The run loop: steps, instructions, gateways.

use crate::calls::HostCalls;
use crate::nested::Nested;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::{GATE_BASE, GATE_END};
use reva_core::mem::PAGE_SIZE;
use reva_core::x86::{Arith, EAX, ESI, ESP, Exec, decode};
use reva_core::{Flow, Gate, Next, Status};
use reva_core_inspect::words::Dictionary;

reva_macros::machine_trait! {
    /// Running instructions and gateways.
    pub trait Run {
        /// Run up to `budget` steps (one instruction or one gateway each).
        fn run(&mut self, host: &mut dyn Host, budget: u64) -> Status {
            for _ in 0..budget {
                if let Some(s) = self.stopped() {
                    return s;
                }
                if std::mem::take(&mut self.interrupt) {
                    self.input.clear();
                    self.recover(host, Fault::Interrupted);
                    continue;
                }
                if let Some(s) = self.after_step(host) {
                    return s;
                }
            }
            match self.halted {
                Some(code) => Status::Halted(code),
                None if self.restart => Status::Restart,
                None => Status::Yielded,
            }
        }

        /// One step, and what it means for `run`: a status to return, or
        /// `None` to go on (faults are reported and recovered from).
        #[inline(always)]
        fn after_step(&mut self, host: &mut dyn Host) -> Option<Status> {
            match self.step(host) {
                Ok(Flow::Continue) => None,
                Ok(Flow::NeedInput) => Some(Status::NeedInput),
                Ok(Flow::HostCall) => Some(Status::HostCall),
                Ok(Flow::Halt(code)) => {
                    self.halted = Some(code);
                    Some(Status::Halted(code))
                }
                Ok(Flow::Restart) => {
                    self.restart = true;
                    Some(Status::Restart)
                }
                Ok(Flow::NestedReturn) => {
                    self.recover(host, Fault::BadGate { addr: Gate::NestedReturn.addr() });
                    None
                }
                Err(f) => {
                    self.recover(host, f);
                    None
                }
            }
        }

        /// One instruction (decoded once, then from the cache) or gateway.
        #[inline(always)]
        fn step(&mut self, host: &mut dyn Host) -> Result<Flow, Fault> {
            self.steps += 1;
            let eip = self.cpu.eip;
            if self.trace {
                eprintln!(
                    "{eip:#010x} {:<22} eax={:08x} esi={:08x} esp={:08x}",
                    self.describe_addr(eip),
                    self.cpu.r[EAX as usize],
                    self.cpu.r[ESI as usize],
                    self.cpu.r[ESP as usize]
                );
            }
            if (GATE_BASE..GATE_END).contains(&eip) {
                return self.enter_gate(host, eip);
            }
            if self.mem.has_dirty() {
                self.invalidate_dirty();
            }
            let insn = match self.cache.get(&eip) {
                Some(i) => *i,
                None => {
                    let i = decode(&self.mem, eip)?;
                    self.mem.mark_code(eip);
                    self.mem.mark_code(eip.wrapping_add(i.len as u32 - 1));
                    self.cache.insert(eip, i);
                    i
                }
            };
            self.cpu.exec(&mut self.mem, &insn)?;
            Ok(Flow::Continue)
        }

        /// Drop cached and translated code of pages that were written.
        fn invalidate_dirty(&mut self) {
            for page in self.mem.take_dirty() {
                // An instruction (max 15 bytes) may start before the page.
                for a in page.saturating_sub(15)..page + PAGE_SIZE {
                    self.cache.remove(&a);
                }
                if let Some(t) = self.translator.as_mut() {
                    t.invalidate(page, PAGE_SIZE);
                }
            }
        }

        /// Run the gate at `addr` and do what it asks next.
        fn enter_gate(&mut self, host: &mut dyn Host, addr: u32) -> Result<Flow, Fault> {
            let g = Gate::from_addr(addr).ok_or(Fault::BadGate { addr })?;
            match (self.dispatch)(self, g, host)? {
                Next::Ret => self.cpu.eip = self.cpu.pop32(&self.mem)?,
                Next::Jump(t) => self.cpu.eip = t,
                Next::Call(t, k) => {
                    self.cpu.push32(&mut self.mem, k.addr())?;
                    self.cpu.eip = t;
                }
                Next::NeedInput => return Ok(Flow::NeedInput),
                Next::Halt(c) => return Ok(Flow::Halt(c)),
                Next::Restart => return Ok(Flow::Restart),
                Next::NestedReturn => return Ok(Flow::NestedReturn),
                Next::HostCall => return Ok(Flow::HostCall),
            }
            Ok(Flow::Continue)
        }
    }
}
