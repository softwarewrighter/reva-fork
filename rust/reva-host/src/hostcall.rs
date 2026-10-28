//! # Host calls: virtual-library functions the host answers
//!
//! Some virtual libraries need something the VM does not have (SQLite, a
//! key-value store on the page).  Their functions become *host calls*
//! (rust/DESIGN.md 3.11): the VM reads the arguments out of its memory into
//! a [`HostCall`] (integers and byte strings, never VM addresses) and asks
//! [`crate::Host::host_call`].  A host that can answer at once (the CLI)
//! returns the reply; one that must ask elsewhere first (the desktop's
//! worker, which posts to the page) returns [`HostAnswer::Later`], and
//! `reva_vm::Machine::run` stops with `reva_vm::Status::HostCall`.  The host
//! then reads `reva_vm::Machine::pending_host_call`, gets the answer, hands
//! it to `reva_vm::Machine::resume` and runs again: the call is retried and
//! takes the reply, as a gateway waiting for input retries when input
//! arrives.  Nothing is popped from the data stack until the reply is in.

/// A call for the host: which service and operation, with the integer
/// arguments and the byte strings (text, blobs) in the order the Forth
/// library passes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCall {
    pub service: &'static str,
    pub op: &'static str,
    pub ints: Vec<i64>,
    pub bytes: Vec<Vec<u8>>,
}

/// The answer to a host call: a number, or bytes (which the VM copies into
/// its memory and returns as their length; see the library's `ptr`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostReply {
    Int(i64),
    Bytes(Vec<u8>),
}

/// What [`crate::Host::host_call`] says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostAnswer {
    /// The answer, now.
    Reply(HostReply),
    /// The answer comes later, through `reva_vm::Machine::resume`.
    Later,
    /// This host does not have the service; the line stops with this
    /// explanation, as for a native library the core does not serve.
    Unavailable(String),
}
