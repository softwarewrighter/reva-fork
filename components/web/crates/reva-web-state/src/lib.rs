//! # reva-web-state — the browser module's state
//!
//! The one VM, its buffer host and the module's buffers ([`state`]), the
//! file system before and after boot ([`buffers`]), and making and
//! remaking the machine ([`start`]).  reva-web-run and reva-web-io export
//! the C ABI over it; reva-web is the module.

pub mod buffers;
pub mod start;
pub mod state;
