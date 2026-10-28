//! # reva-host — the host side of the Reva core
//!
//! What a host (the CLI, the web module, the desktop app) and the VM agree
//! on, with no VM inside: the [`host::Host`] trait and a buffer host, host
//! calls ([`hostcall`]), the virtual file system ([`vfs`]), the sources
//! compiled into the binary ([`embedded`]: `reva.f`, `lib/`), this build's
//! [`version`], and character encodings ([`charset`]).  reva-vm re-exports
//! every module under its old path.

pub mod charset;
pub mod embedded;
pub mod host;
pub mod hostcall;
pub mod version;
pub mod vfs;

pub use host::{BufferHost, Host};
