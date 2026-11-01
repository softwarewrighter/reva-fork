//! Output, synchronous input, diagnostics, colour.

/// The console side of a host.
pub trait Console {
    /// Output from `emit` and `type`.
    fn write(&mut self, bytes: &[u8]);

    /// Input needed *synchronously*: Forth code called from inside a Rust
    /// routine (a "nested call", e.g. a `do_cr` hook) asked for a key and the
    /// queue is empty.  A blocking host (CLI) returns more bytes; a
    /// non-blocking host (browser) returns `None` and the VM faults with
    /// `Fault::InputInNestedCall`.
    fn read_input_blocking(&mut self) -> Option<Vec<u8>> {
        None
    }

    /// A diagnostic from the VM itself, not from Forth code (one line,
    /// newline included), such as a note that a native library is not
    /// available.  The default mixes it into the output; the CLI sends it
    /// to stderr.
    fn diag(&mut self, line: &str) {
        self.write(line.as_bytes());
    }

    /// Whether output may carry ANSI colour escapes (`pp`, `hl-type`; see
    /// reva-vlib's `highlight`).  The default (`false`) keeps output plain;
    /// the CLI says yes at a terminal, the browser page always.
    fn color(&mut self) -> bool {
        false
    }
}
