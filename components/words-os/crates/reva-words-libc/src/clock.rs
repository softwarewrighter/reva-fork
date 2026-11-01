//! `time`, `gettimeofday`, `nanosleep` over the host's clock.

use reva_core::fault::Fault;
use reva_core::host::Host;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// The clock system calls.
    pub trait Clock {
        /// `time(t)`: seconds, also stored at `t` unless it is NULL.
        fn sys_time(&mut self, host: &mut dyn Host, t: u32) -> R<u32> {
            let secs = host.now().map_or(0, |(s, _)| s as u32);
            if t != 0 {
                self.mem.write_u32(t, secs)?;
            }
            Ok(secs)
        }

        /// `gettimeofday(tv, tz)`: seconds and microseconds at `tv`.
        fn sys_timeofday(&mut self, host: &mut dyn Host, tv: u32) -> R<u32> {
            let (s, us) = host.now().unwrap_or((0, 0));
            if tv != 0 {
                self.mem.write_u32(tv, s as u32)?;
                self.mem.write_u32(tv + 4, us)?;
            }
            Ok(0)
        }

        /// `nanosleep(req, rem)`: the host sleeps.
        fn sys_sleep(&mut self, host: &mut dyn Host, req: u32) -> R<u32> {
            if req != 0 {
                let s = self.mem.read_u32(req)?;
                let ns = self.mem.read_u32(req + 4)?;
                host.sleep_ms(s.saturating_mul(1000).saturating_add(ns / 1_000_000));
            }
            Ok(0)
        }
    }
}
