//! The environment, the clock, sleeping, the shell.

/// The operating-system side of a host; all of it optional.
pub trait Platform {
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

    /// The C library's `system(cmd)`, behind Reva's `shell`: run an OS
    /// command and return its wait status.  `None` (the default) means the
    /// host does not allow it, and the VM faults with a message saying so;
    /// the CLI allows it only with `--allow-shell`, and the browser never.
    fn system(&mut self, _cmd: &str) -> Option<i32> {
        None
    }
}
