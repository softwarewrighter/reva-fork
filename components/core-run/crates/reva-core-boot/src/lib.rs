//! # reva-core-boot — building machines
//!
//! A fresh machine ([`boot::Boot::new`]): its memory, statics and core
//! dictionary ([`tables`]), `argc`/`(argv)`, the registers at `cold`; or
//! one recreated from an image ([`image`], whose format errors are in
//! [`format`]).

pub mod boot;
pub mod format;
pub mod image;
pub mod tables;
