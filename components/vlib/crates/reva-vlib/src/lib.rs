//! # reva-vlib — virtual libraries of the Reva core
//!
//! Rust in place of the C libraries Reva programs load with `lib` and
//! `func:` (docs/design.md 3.11, docs/portability.md): `highlight`,
//! `libiconv`, `reva-sqlite` (host calls) and `turtle`, as extension
//! traits of reva-core's `Machine`.

pub mod highlight;
pub mod iconv;
pub mod sqlite;
pub mod turtle;

// Paths the moved code uses (crate::x86, crate::host, ...), from the
// crates they live in.
pub use reva_core::{Machine, Next, st};
pub use reva_core::{canvas, gate, gates, machine};
pub use reva_host::{charset, embedded, host, hostcall, version, vfs};
pub use reva_x86 as x86;
pub use reva_x86::{fault, layout, mem};
