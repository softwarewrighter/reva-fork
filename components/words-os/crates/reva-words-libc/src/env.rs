//! `getenv`, `system`, `putenv`, `localtime`.

use crate::time::tm_from_unix;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// The virtual libc's environment, shell and clock functions.
    pub trait Env {
        /// `char *getenv(const char *)`: the value, zero-terminated, in a
        /// buffer reused by the next call (as C allows), or NULL.
        fn c_getenv(&mut self, host: &mut dyn Host, name: u32) -> R<u32> {
            let name = self.ffi_zstring(name)?;
            let Some(value) = host.getenv(&name) else {
                return Ok(0);
            };
            let bytes = value.as_bytes();
            let p = self.ffi_buffer(st::ENV_BUF, Some(st::ENV_LEN), bytes.len() as u32 + 1)?;
            self.mem.write_bytes(p, bytes)?;
            self.mem.write_u8(p + bytes.len() as u32, 0)?;
            Ok(p)
        }

        /// `int system(const char *)`: the host runs the command if it allows
        /// that, else the call faults with how to allow it.  `system(NULL)`
        /// asks whether a shell exists: 0 here, as nothing is promised.
        fn c_system(&mut self, host: &mut dyn Host, cmd: u32) -> R<u32> {
            if cmd == 0 {
                return Ok(0);
            }
            let cmd = self.ffi_zstring(cmd)?;
            match host.system(&cmd) {
                Some(status) => Ok(status as u32),
                None => Err(Fault::NotPermitted {
                    what: "shell (running an OS command)",
                    hint: "the CLI allows it with --allow-shell; the browser never does",
                }),
            }
        }

        /// `int putenv(char *)` with `name=value`.
        fn c_putenv(&mut self, host: &mut dyn Host, s: u32) -> R<u32> {
            let s = self.ffi_zstring(s)?;
            match s.split_once('=') {
                Some((k, v)) => {
                    host.setenv(k, v);
                    Ok(0)
                }
                None => Ok(u32::MAX),
            }
        }

        /// `struct tm *localtime(const time_t *)`.  The port has no time-zone
        /// database, so local time is UTC (documented in docs/gotchas.md).
        fn c_localtime(&mut self, tp: u32) -> R<u32> {
            let t = self.mem.read_u32(tp)? as i32 as i64;
            let tm = tm_from_unix(t);
            let p = self.ffi_buffer(st::TM_BUF, None, 11 * 4)?;
            for (i, v) in tm.iter().enumerate() {
                self.mem.write_u32(p + 4 * i as u32, *v as u32)?;
            }
            self.mem.write_u32(p + 36, 0)?; // tm_gmtoff
            self.mem.write_u32(p + 40, 0)?; // tm_zone
            Ok(p)
        }
    }
}
