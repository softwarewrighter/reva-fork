//! Fault messages.

use crate::class::insn_class;
use crate::fault::{Fault, PORTABILITY_DOC};
use std::fmt;

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Fault::BadOpcode { eip, bytes } = self {
            return bad_opcode(f, *eip, bytes);
        }
        f.write_str(&self.message())
    }
}

impl std::error::Error for Fault {}

/// "unsupported CLASS instruction at EIP: BYTES (see the doc)".
fn bad_opcode(f: &mut fmt::Formatter<'_>, eip: u32, bytes: &[u8]) -> fmt::Result {
    let class = insn_class(bytes);
    write!(f, "unsupported {class} instruction at {eip:#010x}:")?;
    for b in bytes {
        write!(f, " {b:02x}")?;
    }
    write!(f, " (see {PORTABILITY_DOC})")
}

impl Fault {
    /// The message of any fault but `BadOpcode`.
    fn message(&self) -> String {
        self.machine_message()
            .unwrap_or_else(|| self.host_message())
    }

    /// Faults of the machine itself: memory, arithmetic, control.
    fn machine_message(&self) -> Option<String> {
        Some(match self {
            Fault::BadAddress { addr, len } => {
                format!("bad memory access: {len} byte(s) at {addr:#010x}")
            }
            Fault::DivideError { eip } => format!("divide error at {eip:#010x}"),
            Fault::BadGate { addr } => format!("no core routine at {addr:#010x}"),
            Fault::Trap { eip, what } => format!("{what} at {eip:#010x}"),
            Fault::StackUnderflow => "stack underflow".into(),
            Fault::ReturnStackOverflow => "return stack overflow".into(),
            Fault::OutOfSpace(what) => format!("out of {what} space"),
            Fault::Interrupted => "interrupted".into(),
            _ => return None,
        })
    }

    /// Faults of the port and the host: what is missing or not allowed.
    fn host_message(&self) -> String {
        match self {
            Fault::NotImplemented(w) => format!("'{w}' is not implemented in this port yet"),
            Fault::InputInNestedCall => {
                "input requested inside a host callback (not resumable)".into()
            }
            Fault::UnwoundThroughHost => "throw unwound through a host callback".into(),
            Fault::NativeCall { addr } => format!(
                "call to native code at {addr:#010x}: the portable core runs no native \
                 libraries (see {PORTABILITY_DOC})"
            ),
            Fault::Unavailable(why) => format!("{why} (see {PORTABILITY_DOC})"),
            Fault::NotPermitted { what, hint } => {
                format!("{what} is not allowed here: {hint} (see {PORTABILITY_DOC})")
            }
            // the machine's faults (machine_message) and BadOpcode (fmt)
            _ => String::new(),
        }
    }
}
