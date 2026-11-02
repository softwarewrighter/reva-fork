//! A new file, never over an existing one.

use std::fs::OpenOptions;
use std::io::{self, ErrorKind, Write};
use std::path::Path;

/// Write `bytes` to the first free `STEM-NNN.EXT` in `dir` (001, 002, ...,
/// more digits past 999); its name.  The file is created exclusively, so
/// an existing file (or one made meanwhile) is never overwritten.
pub fn save_new(dir: &Path, stem: &str, ext: &str, bytes: &[u8]) -> io::Result<String> {
    for n in 1u64.. {
        let name = format!("{stem}-{n:03}.{ext}");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(&name))
        {
            Ok(mut f) => {
                f.write_all(bytes)?;
                return Ok(name);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    unreachable!("u64 file numbers run out")
}
