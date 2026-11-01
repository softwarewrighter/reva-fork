//! The buffer host's state.

use reva_memfs::memfs::MemFs;
use std::collections::HashMap;

/// A host that captures output in memory, with an in-memory file system
/// ([`MemFs`]).  Used by tests and by the WebAssembly front end.
#[derive(Default)]
pub struct BufferHost {
    pub out: Vec<u8>,
    pub fs: MemFs,
    pub env: HashMap<String, String>,
    /// What `now` reports: set by the embedder (the browser sets it before
    /// each run); `None` (the default) keeps tests deterministic.
    pub clock: Option<(u64, u32)>,
    /// What `color` reports (the browser sets it; tests choose).
    pub color: bool,
    /// Answer host calls later (`HostAnswer::Later`): the embedder (the web
    /// module's page) answers them.  Off: none.
    pub host_calls: bool,
}

impl BufferHost {
    pub fn new() -> BufferHost {
        BufferHost::default()
    }

    /// Take everything written so far as a (lossy) string.
    pub fn take_output(&mut self) -> String {
        String::from_utf8_lossy(&std::mem::take(&mut self.out)).into_owned()
    }
}
