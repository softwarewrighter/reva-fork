//! # reva-host-buffer — a host in memory
//!
//! [`buffer::BufferHost`]: output captured in memory, an in-memory file
//! system (reva-memfs), an environment and a clock the embedder sets.
//! Used by tests and by the WebAssembly front end.

pub mod buffer;
pub mod console;
pub mod platform;
pub mod resources;
