//! # reva-hostpath — host paths for the VM
//!
//! Where the virtual install prefix `/reva/` lives, VM paths to host paths
//! and back (Windows drives), `io::Error`s as Linux `errno`s, metadata as
//! the VM's `Stat`.

pub mod paths;
