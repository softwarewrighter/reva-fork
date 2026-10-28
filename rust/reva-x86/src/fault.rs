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

/// What kind of x86 code `bytes` (an instruction, prefixes first) is, for
/// fault messages: names the family the portable core leaves out, so a
/// reader knows at once whether it is floating point, a vector extension,
/// a privileged instruction or a plain gap in the decoder.
pub fn insn_class(bytes: &[u8]) -> &'static str {
    let mut i = 0;
    let mut sse_prefix = false;
    while let Some(&b) = bytes.get(i) {
        match b {
            0x26 | 0x2e | 0x36 | 0x3e | 0x64 | 0x65 => return "segment override",
            0xf0 => return "lock prefix",
            0x66 | 0xf2 | 0xf3 => {
                sse_prefix = true;
                i += 1;
            }
            _ => break,
        }
    }
    let Some(&op) = bytes.get(i) else {
        return "x86";
    };
    match op {
        0xd8..=0xdf | 0x9b => "x87 floating point",
        0xe4..=0xe7 | 0xec..=0xef | 0x6c..=0x6f => "I/O port (privileged)",
        0xfa | 0xfb | 0xf4 | 0xcf => "privileged",
        0x8e | 0x8c | 0x06 | 0x07 | 0x0e | 0x16 | 0x17 | 0x1e | 0x1f => "segment register",
        0xcc..=0xce => "software interrupt",
        0x0f => match bytes.get(i + 1).copied().unwrap_or(0) {
            0x00 | 0x01 | 0x06 | 0x08 | 0x09 | 0x20..=0x23 | 0x30 | 0x32 | 0x33 => "privileged",
            0x0b => "undefined instruction (ud2)",
            0xa0 | 0xa1 | 0xa8 | 0xa9 => "segment register",
            0xa2 | 0x31 => "processor identification (cpuid/rdtsc)",
            0x10..=0x17 | 0x28..=0x2f | 0x50..=0x5f | 0xc2 | 0xc6 => "SSE",
            0x60..=0x7f | 0xd0..=0xff if sse_prefix => "SSE2",
            0x60..=0x7f | 0xd0..=0xff => "MMX",
            _ => "x86",
        },
        _ => "x86",
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::BadAddress { addr, len } => {
                write!(f, "bad memory access: {len} byte(s) at {addr:#010x}")
            }
            Fault::BadOpcode { eip, bytes } => {
                let class = insn_class(bytes);
                write!(f, "unsupported {class} instruction at {eip:#010x}:")?;
                for b in bytes {
                    write!(f, " {b:02x}")?;
                }
                write!(f, " (see {PORTABILITY_DOC})")
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
            Fault::Interrupted => write!(f, "interrupted"),
            Fault::NativeCall { addr } => write!(
                f,
                "call to native code at {addr:#010x}: the portable core runs no native \
                 libraries (see {PORTABILITY_DOC})"
            ),
            Fault::Unavailable(why) => write!(f, "{why} (see {PORTABILITY_DOC})"),
            Fault::NotPermitted { what, hint } => {
                write!(
                    f,
                    "{what} is not allowed here: {hint} (see {PORTABILITY_DOC})"
                )
            }
        }
    }
}

impl std::error::Error for Fault {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_classes() {
        assert_eq!(insn_class(&[0xd9, 0xe8]), "x87 floating point"); // fld1
        assert_eq!(insn_class(&[0x0f, 0x6f, 0xc1]), "MMX"); // movq mm0, mm1
        assert_eq!(insn_class(&[0x66, 0x0f, 0x6f, 0xc1]), "SSE2"); // movdqa
        assert_eq!(insn_class(&[0x0f, 0x28, 0xc1]), "SSE"); // movaps
        assert_eq!(insn_class(&[0xe4, 0x60]), "I/O port (privileged)"); // in al, 0x60
        assert_eq!(insn_class(&[0xfa]), "privileged"); // cli
        assert_eq!(insn_class(&[0x64, 0x8b, 0x00]), "segment override"); // mov eax, fs:[eax]
        assert_eq!(
            insn_class(&[0x0f, 0xa2]),
            "processor identification (cpuid/rdtsc)"
        );
        assert_eq!(insn_class(&[0x0f, 0x0b]), "undefined instruction (ud2)");
        assert_eq!(insn_class(&[0xd6]), "x86");
    }

    #[test]
    fn bad_opcode_message_names_the_class_and_the_doc() {
        let f = Fault::BadOpcode {
            eip: 0x2000,
            bytes: vec![0xd9, 0xe8],
        };
        assert_eq!(
            f.to_string(),
            "unsupported x87 floating point instruction at 0x00002000: d9 e8 \
             (see docs/portability.md)"
        );
    }
}
