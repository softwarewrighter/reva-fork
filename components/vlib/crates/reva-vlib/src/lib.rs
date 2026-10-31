//! # reva-vlib — virtual libraries of the Reva core
//!
//! Rust in place of the C libraries Reva programs load with `lib` and
//! `func:` (docs/design.md 3.11, docs/portability.md): `highlight`,
//! `libiconv`, `reva-sqlite` (host calls) and `turtle`, each its own crate
//! (extension traits of reva-core's `Machine`), here under their old paths.

pub use reva_vlib_highlight as highlight;
pub use reva_vlib_iconv::iconv;
pub use reva_vlib_sqlite::sqlite;
pub use reva_vlib_turtle as turtle;
