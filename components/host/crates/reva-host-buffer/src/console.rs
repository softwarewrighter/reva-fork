//! Output into the buffer.

use crate::buffer::BufferHost;
use reva_host_traits::console::Console;

impl Console for BufferHost {
    fn write(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    fn color(&mut self) -> bool {
        self.color
    }
}
