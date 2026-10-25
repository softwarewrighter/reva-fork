//! # Starting, and starting again
//!
//! How a host made its machine: booting a source file (normally the
//! embedded `reva.f`), or loading an image.  A host keeps its [`Start`] so
//! that `restart` ([`Status::Restart`]) can make the machine again exactly
//! the same way, as Reva's own `restart` relaunched the executable.  What
//! the host owns survives a restart: its files and its output.
//!
//! [`Status::Restart`]: crate::Status::Restart

use crate::image::ImageError;
use crate::machine::Machine;

#[derive(Clone, Debug)]
pub enum Start {
    /// [`Machine::new`] with these `argv` (`args[1]`, if present, is the
    /// file the core includes, e.g. [`crate::embedded::BOOT_PATH`]).
    Boot(Vec<String>),
    /// [`Machine::load_image`] of these bytes, with these `argv`.
    Image(Vec<u8>, Vec<String>),
}

impl Start {
    /// Boot `reva.f` (embedded), as `reva` with no options does.
    pub fn reva() -> Start {
        Start::Boot(vec!["reva".into(), crate::embedded::BOOT_PATH.into()])
    }

    /// Start from an image; fails now, not at the first restart, if the
    /// bytes are not an image this core can load.
    pub fn image(bytes: Vec<u8>) -> Result<(Start, Machine), ImageError> {
        let start = Start::Image(bytes, vec!["reva".into()]);
        let m = start.machine()?;
        Ok((start, m))
    }

    /// A new machine, ready to [`Machine::run`].
    pub fn machine(&self) -> Result<Machine, ImageError> {
        fn argv(a: &[String]) -> Vec<&str> {
            a.iter().map(String::as_str).collect()
        }
        match self {
            Start::Boot(args) => Ok(Machine::new(&argv(args))),
            Start::Image(bytes, args) => Machine::load_image(bytes, &argv(args)),
        }
    }
}
