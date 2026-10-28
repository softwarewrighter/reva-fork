//! The host interface: everything the VM needs from the outside world.
//!
//! The VM never blocks.  Input is *pushed* into it (`reva_vm::Machine::feed`)
//! and `reva_vm::Machine::run` returns `reva_vm::Status::NeedInput` when Forth
//! code wants a key that has not arrived.  That keeps one core usable from a
//! blocking CLI loop and from a browser event loop alike.

use std::collections::HashMap;

use crate::vfs::{FileSystem, MemFs};

pub trait Host {
    /// Output from `emit` and `type`.
    fn write(&mut self, bytes: &[u8]);

    /// Sources the host serves for `include` and `slurp` *before* the file
    /// system ([`Host::fs`]): the embedded boot source, normally
    /// ([`crate::embedded::file`]).  `None` means "ask the file system".
    fn read_file(&mut self, path: &str) -> Option<Vec<u8>> {
        crate::embedded::file(path).map(<[u8]>::to_vec)
    }

    /// The file system behind `open/r`, `read`, `stat`, ... (see
    /// [`crate::vfs`]).  `None` (the default): every file operation fails
    /// with `ENOSYS`, and `include` finds only [`Host::read_file`] sources.
    fn fs(&mut self) -> Option<&mut dyn FileSystem> {
        None
    }

    /// Input needed *synchronously*: Forth code called from inside a Rust
    /// routine (a "nested call", e.g. a `do_cr` hook) asked for a key and the
    /// queue is empty.  A blocking host (CLI) returns more bytes; a
    /// non-blocking host (browser) returns `None` and the VM faults with
    /// `reva_vm::Fault::InputInNestedCall`.
    fn read_input_blocking(&mut self) -> Option<Vec<u8>> {
        None
    }

    /// Environment lookup for `getenv`.  `None`: not set (the default, so a
    /// host without an environment, like the browser, needs nothing).
    fn getenv(&mut self, _name: &str) -> Option<String> {
        None
    }

    /// `setenv`/`putenv`.  The default ignores it.
    fn setenv(&mut self, _name: &str, _value: &str) {}

    /// Wall-clock time as (seconds, microseconds) since the Unix epoch, for
    /// `time&date`, `ms@` and friends.  `None` (the default) reads as 0, so
    /// the VM itself never touches a clock and stays deterministic.
    fn now(&mut self) -> Option<(u64, u32)> {
        None
    }

    /// Answer a virtual-library host call (see [`crate::hostcall`]): now,
    /// later through `reva_vm::Machine::resume`, or not at all.  The
    /// default has no services.
    fn host_call(&mut self, call: &crate::hostcall::HostCall) -> crate::hostcall::HostAnswer {
        crate::hostcall::HostAnswer::Unavailable(format!(
            "{} is not available on this host",
            call.service
        ))
    }

    /// `ms`: pause for `ms` milliseconds.  The default returns at once (a
    /// browser cannot block).
    fn sleep_ms(&mut self, _ms: u32) {}

    /// A diagnostic from the VM itself, not from Forth code (one line,
    /// newline included), such as a note that a native library is not
    /// available.  The default mixes it into the output; the CLI sends it
    /// to stderr.
    fn diag(&mut self, line: &str) {
        self.write(line.as_bytes());
    }

    /// The C library's `system(cmd)`, behind Reva's `shell`: run an OS
    /// command and return its wait status.  `None` (the default) means the
    /// host does not allow it, and the VM faults with a message saying so;
    /// the CLI allows it only with `--allow-shell`, and the browser never.
    fn system(&mut self, _cmd: &str) -> Option<i32> {
        None
    }

    /// Whether output may carry ANSI colour escapes (`pp`, `hl-type`; see
    /// `reva_vm::highlight`).  The default (`false`) keeps output plain;
    /// the CLI says yes at a terminal, the browser page always.
    fn color(&mut self) -> bool {
        false
    }
}

/// A host that captures output in memory, with an in-memory file system
/// ([`MemFs`]).  Used by tests and by the WebAssembly front end.
#[derive(Default)]
pub struct BufferHost {
    pub out: Vec<u8>,
    pub fs: MemFs,
    pub env: HashMap<String, String>,
    /// What `now` reports: set by the embedder (the browser sets it before
    /// each run); `None` (the default) keeps tests deterministic.
    pub clock: Option<(u64, u32)>,
    /// What [`Host::color`] reports (the browser sets it; tests choose).
    pub color: bool,
    /// Answer host calls later ([`crate::hostcall::HostAnswer::Later`]):
    /// the embedder (the web module's page) answers them.  Off: none.
    pub host_calls: bool,
}

impl BufferHost {
    pub fn new() -> BufferHost {
        BufferHost::default()
    }

    /// Take everything written so far as a (lossy) string.
    pub fn take_output(&mut self) -> String {
        String::from_utf8_lossy(&std::mem::take(&mut self.out)).into_owned()
    }
}

impl Host for BufferHost {
    fn write(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    fn fs(&mut self) -> Option<&mut dyn FileSystem> {
        Some(&mut self.fs)
    }

    fn host_call(&mut self, call: &crate::hostcall::HostCall) -> crate::hostcall::HostAnswer {
        if self.host_calls {
            return crate::hostcall::HostAnswer::Later;
        }
        crate::hostcall::HostAnswer::Unavailable(format!(
            "{} is not available on this host",
            call.service
        ))
    }

    fn getenv(&mut self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }

    fn now(&mut self) -> Option<(u64, u32)> {
        self.clock
    }

    fn color(&mut self) -> bool {
        self.color
    }

    fn setenv(&mut self, name: &str, value: &str) {
        self.env.insert(name.to_string(), value.to_string());
    }
}
