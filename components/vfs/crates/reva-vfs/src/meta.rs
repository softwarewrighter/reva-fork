//! What a path or descriptor is, and changing names.

use crate::types::{Errno, Stat};

/// Status, removal, renaming, new directories.
pub trait FsMeta {
    fn fstat(&mut self, fd: u32) -> Result<Stat, Errno>;
    fn stat(&mut self, path: &str) -> Result<Stat, Errno>;
    /// `unlink`.
    fn remove(&mut self, path: &str) -> Result<(), Errno>;
    fn rename(&mut self, from: &str, to: &str) -> Result<(), Errno>;
    fn mkdir(&mut self, path: &str) -> Result<(), Errno>;
}
