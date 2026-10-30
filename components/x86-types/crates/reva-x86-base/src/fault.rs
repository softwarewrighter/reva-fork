//! Faults: everything that can stop a VM step.
//!
//! The original core relied on the OS to deliver SIGSEGV/SIGFPE and then
//! jumped to the `exception` vector.  The portable core detects the same
//! conditions explicitly and reports them with enough context (address,
//! instruction bytes, routine name) to find the cause.

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
    /// The host asked to stop the running code (`reva_vm::Machine::interrupt`).
    Interrupted,
    /// `(call)` of an address that is not a virtual-library function: the
    /// portable core never runs native code (see docs/portability.md).
    NativeCall { addr: u32 },
    /// An OS service the host does not allow, with what would enable it.
    NotPermitted {
        what: &'static str,
        hint: &'static str,
    },
    /// A host call this host cannot answer (a service it does not have).
    Unavailable(String),
}

/// Where unsupported code is explained.
pub const PORTABILITY_DOC: &str = "docs/portability.md";
