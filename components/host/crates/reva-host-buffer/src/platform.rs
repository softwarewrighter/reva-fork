//! The embedder's environment and clock.

use crate::buffer::BufferHost;
use reva_host_traits::platform::Platform;

impl Platform for BufferHost {
    fn getenv(&mut self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }

    fn setenv(&mut self, name: &str, value: &str) {
        self.env.insert(name.to_string(), value.to_string());
    }

    fn now(&mut self) -> Option<(u64, u32)> {
        self.clock
    }
}
