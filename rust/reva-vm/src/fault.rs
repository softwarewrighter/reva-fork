//! Faults: everything that can stop a VM step.
//!
//! The original core relied on the OS to deliver SIGSEGV/SIGFPE and then
//! jumped to the `exception` vector.  The portable core detects the same
//! conditions explicitly and reports them with enough context (address,
//! instruction bytes, routine name) to find the cause.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// Memory access outside VM memory or inside the null guard.
    BadAddress { addr: u32, len: u32 },
    /// An instruction outside the supported IA-32 subset.
    BadOpcode { eip: u32, bytes: Vec<u8> },
    /// `div`/`idiv` by zero or with a quotient that does not fit.
    DivideError { eip: u32 },
    /// Execution reached a gateway slot with no routine behind it.
    BadGate { addr: u32 },
    /// Execution of `hlt` or `int3`.
    Trap { eip: u32, what: &'static str },
    /// The data stack pointer moved above `S0`.
    StackUnderflow,
    /// `push`/`call` would move ESP below the return-stack region.
    ReturnStackOverflow,
    /// A core word that this port does not provide yet.
    NotImplemented(&'static str),
    /// A nested call (from a Rust routine into Forth) needed input that the
    /// host could not supply synchronously.
    InputInNestedCall,
    /// `throw` unwound past a Rust routine that had called into Forth.
    UnwoundThroughHost,
    /// A region (code, dictionary, heap) is full.
    OutOfSpace(&'static str),
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::BadAddress { addr, len } => {
                write!(f, "bad memory access: {len} byte(s) at {addr:#010x}")
            }
            Fault::BadOpcode { eip, bytes } => {
                write!(f, "unsupported instruction at {eip:#010x}:")?;
                for b in bytes {
                    write!(f, " {b:02x}")?;
                }
                Ok(())
            }
            Fault::DivideError { eip } => write!(f, "divide error at {eip:#010x}"),
            Fault::BadGate { addr } => write!(f, "no core routine at {addr:#010x}"),
            Fault::Trap { eip, what } => write!(f, "{what} at {eip:#010x}"),
            Fault::StackUnderflow => write!(f, "stack underflow"),
            Fault::ReturnStackOverflow => write!(f, "return stack overflow"),
            Fault::NotImplemented(w) => write!(f, "'{w}' is not implemented in this port yet"),
            Fault::InputInNestedCall => {
                write!(f, "input requested inside a host callback (not resumable)")
            }
            Fault::UnwoundThroughHost => write!(f, "throw unwound through a host callback"),
            Fault::OutOfSpace(what) => write!(f, "out of {what} space"),
        }
    }
}

impl std::error::Error for Fault {}
