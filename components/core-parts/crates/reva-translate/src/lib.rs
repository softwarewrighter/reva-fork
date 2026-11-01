//! # reva-translate — hooks for translators, and the decode cache
//!
//! [`translate`]: the contract a native or WebAssembly translator meets;
//! [`cache`]: the per-address cache of decoded instructions.

pub mod cache;
pub mod translate;
