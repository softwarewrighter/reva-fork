//! The host interface: everything the VM needs from the outside world.
//!
//! The VM never blocks.  Input is *pushed* into it ([`crate::Machine::feed`])
//! and [`crate::Machine::run`] returns [`crate::Status::NeedInput`] when Forth
//! code wants a key that has not arrived.  That keeps one core usable from a
//! blocking CLI loop and from a browser event loop alike.

use std::collections::HashMap;

pub trait Host {
    /// Output from `emit` and `type`.
    fn write(&mut self, bytes: &[u8]);

    /// Whole-file read for `include` and `slurp`.  `None` means "not found";
    /// the VM sets `ioerr` and Forth code decides what to do.
    fn read_file(&mut self, _path: &str) -> Option<Vec<u8>> {
        None
    }

    /// Input needed *synchronously*: Forth code called from inside a Rust
    /// routine (a "nested call", e.g. a `do_cr` hook) asked for a key and the
    /// queue is empty.  A blocking host (CLI) returns more bytes; a
    /// non-blocking host (browser) returns `None` and the VM faults with
    /// [`crate::Fault::InputInNestedCall`].
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

    /// `ms`: pause for `ms` milliseconds.  The default returns at once (a
    /// browser cannot block).
    fn sleep_ms(&mut self, _ms: u32) {}
}

/// A host that captures output in memory and serves files from a map.
/// Used by tests and by the WebAssembly front end.
#[derive(Default)]
pub struct BufferHost {
    pub out: Vec<u8>,
    pub files: HashMap<String, Vec<u8>>,
    pub env: HashMap<String, String>,
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

    fn read_file(&mut self, path: &str) -> Option<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .or_else(|| crate::embedded::file(path).map(<[u8]>::to_vec))
    }

    fn getenv(&mut self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }

    fn setenv(&mut self, name: &str, value: &str) {
        self.env.insert(name.to_string(), value.to_string());
    }
}
