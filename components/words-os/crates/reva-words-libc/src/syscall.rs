//! `os_syscall`: the arguments, the file calls, the others.

use crate::clock::Clock;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::{EAX, ESI};
use reva_core_ffi::regs::MachineRegs;
use reva_words_files::sysfile::SysFile;

type R<T = ()> = Result<T, Fault>;

// Linux i386 numbers used by `reva.f` outside file I/O.
const SYS_TIME: u32 = 13;
const SYS_GETPID: u32 = 20;
const SYS_IOCTL: u32 = 54;
const SYS_GETTIMEOFDAY: u32 = 78;
const SYS_NANOSLEEP: u32 = 162;
const ENOSYS: u32 = 38;
const ENOTTY: u32 = 25;

reva_macros::machine_trait! {
    /// Linux system calls by number.
    pub trait Syscall: SysFile, Clock {
        /// `os_syscall` ( p1 .. pN N num -- result ): the topmost argument is
        /// the first (EBX), as in the original register loading.
        fn ffi_syscall(&mut self, host: &mut dyn Host) -> R {
            let num = self.reg(EAX);
            let sp = self.reg(ESI);
            let count = self.mem.read_u32(sp)?;
            if count > 6 {
                return Ok(());
            }
            let a = self.syscall_args(sp, count)?;
            let result = match self.sys_file(host, num, &a)? {
                Some(r) => r,
                None => self.sys_other(host, num, &a)?,
            };
            self.set_reg(ESI, sp + 4 + 4 * count);
            self.set_reg(EAX, result);
            Ok(())
        }

        /// The `count` arguments above the count cell at `sp`.
        fn syscall_args(&self, sp: u32, count: u32) -> R<[u32; 6]> {
            let mut a = [0u32; 6];
            for (i, slot) in a.iter_mut().enumerate().take(count as usize) {
                *slot = self.mem.read_u32(sp + 4 + 4 * i as u32)?;
            }
            Ok(a)
        }

        /// The calls other than file calls: the clock, sleep, pid, ioctl.
        fn sys_other(&mut self, host: &mut dyn Host, num: u32, a: &[u32; 6]) -> R<u32> {
            Ok(match num {
                SYS_TIME => self.sys_time(host, a[0])?,
                SYS_GETTIMEOFDAY => self.sys_timeofday(host, a[0])?,
                SYS_NANOSLEEP => self.sys_sleep(host, a[0])?,
                SYS_GETPID => 1,
                SYS_IOCTL => ENOTTY.wrapping_neg(),
                _ => ENOSYS.wrapping_neg(),
            })
        }
    }
}
