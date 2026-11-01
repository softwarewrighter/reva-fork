//! Sources, the file system, host calls; and the host as one trait.

use crate::console::Console;
use crate::platform::Platform;
use reva_embedded::embedded;
use reva_hostcall::hostcall::{HostAnswer, HostCall};
use reva_vfs::dirs::FileSystem;

/// What a host lends the VM.
pub trait Resources {
    /// Sources the host serves for `include` and `slurp` *before* the file
    /// system ([`Resources::fs`]): the embedded boot source, normally
    /// (reva-embedded's `file`).  `None` means "ask the file system".
    fn read_file(&mut self, path: &str) -> Option<Vec<u8>> {
        embedded::file(path).map(<[u8]>::to_vec)
    }

    /// The file system behind `open/r`, `read`, `stat`, ... (see reva-vfs).
    /// `None` (the default): every file operation fails with `ENOSYS`, and
    /// `include` finds only [`Resources::read_file`] sources.
    fn fs(&mut self) -> Option<&mut dyn FileSystem> {
        None
    }

    /// Answer a virtual-library host call (see reva-hostcall): now, later
    /// through reva-core-run's `resume`, or not at all.  The default has no
    /// services.
    fn host_call(&mut self, call: &HostCall) -> HostAnswer {
        HostAnswer::Unavailable(format!("{} is not available on this host", call.service))
    }
}

/// Everything the VM needs from the outside world: any type with the three
/// parts is a host (implement [`Console`], [`Platform`], [`Resources`]).
pub trait Host: Console + Platform + Resources {}

impl<T: Console + Platform + Resources> Host for T {}
