//! # reva-macros — boilerplate as macros, for the Reva core crates
//!
//! `machine_trait!`: methods of reva-core's `Machine` written once in
//! another crate (docs/design.md 3.12).  `words!`: a family's part of the
//! gate dispatch table (reva-words).  The crates using these two depend on
//! reva-core, which they name.  `ext_trait!`: methods of any type defined
//! in another crate (reva-x86-types' `Cpu`, for the executor).

mod ext_trait;
mod machine_trait;
mod words;
