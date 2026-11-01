//! Process environment, clock and the shell.

use std::io::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::host::{StdHost, wait_status};
use reva_vm::host::Platform;

impl Platform for StdHost {
    fn getenv(&mut self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn setenv(&mut self, name: &str, value: &str) {
        // SAFETY: reva-cli runs on one thread (it starts none), so no
        // other thread can read the environment while it changes.
        unsafe { std::env::set_var(name, value) };
    }

    fn now(&mut self) -> Option<(u64, u32)> {
        let d = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
        Some((d.as_secs(), d.subsec_micros()))
    }

    fn sleep_ms(&mut self, ms: u32) {
        std::thread::sleep(Duration::from_millis(ms.into()));
    }

    /// `system(3)`: `sh -c cmd`, returning the wait status as C does.
    /// Only with `--allow-shell`: Reva programs run OS commands freely, so
    /// that is a choice for whoever starts them, not for the program.
    fn system(&mut self, cmd: &str) -> Option<i32> {
        if !self.allow_shell {
            return None;
        }
        let _ = self.out.flush();
        let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();
        Some(match status {
            Ok(s) => wait_status(s),
            Err(_) => 127 << 8,
        })
    }
}
