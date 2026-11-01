//! # reva-host — the host side of the Reva core
//!
//! What a host (the CLI, the web module, the desktop app) and the VM agree
//! on, with no VM inside: the [`host::Host`] traits and a buffer host, host
//! calls ([`hostcall`]), the virtual file system ([`vfs`]), the sources
//! compiled into the binary ([`embedded`]: `reva.f`, `lib/`), this build's
//! [`version`], and character encodings ([`charset`]).  The parts are
//! their own crates (components host-base, vfs, host); this facade keeps
//! their old paths, and reva-vm re-exports them again.

pub mod host;
pub mod vfs;

pub use host::{BufferHost, Host};
pub use reva_charset::charset;
pub use reva_embedded::embedded;
pub use reva_hostcall::hostcall;
pub use reva_version::version;
