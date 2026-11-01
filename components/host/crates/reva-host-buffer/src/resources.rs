//! The in-memory file system, and host calls left to the embedder.

use crate::buffer::BufferHost;
use reva_host_traits::resources::Resources;
use reva_hostcall::hostcall::{HostAnswer, HostCall};
use reva_vfs::dirs::FileSystem;

impl Resources for BufferHost {
    fn fs(&mut self) -> Option<&mut dyn FileSystem> {
        Some(&mut self.fs)
    }

    fn host_call(&mut self, call: &HostCall) -> HostAnswer {
        if self.host_calls {
            return HostAnswer::Later;
        }
        HostAnswer::Unavailable(format!("{} is not available on this host", call.service))
    }
}
