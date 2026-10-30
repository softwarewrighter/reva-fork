//! # reva-host-traits — what the VM needs from the outside world
//!
//! The VM never blocks.  Input is *pushed* into it (reva-core-run's `feed`)
//! and `run` returns `Status::NeedInput` when Forth code wants a key that
//! has not arrived.  That keeps one core usable from a blocking CLI loop
//! and from a browser event loop alike.
//!
//! A host is [`resources::Host`]: [`console::Console`] (output, blocking
//! input, diagnostics, colour), [`platform::Platform`] (environment,
//! clock, sleep, shell) and [`resources::Resources`] (sources, the file
//! system, host calls).  Every method but `write` has a default.

pub mod console;
pub mod platform;
pub mod resources;
