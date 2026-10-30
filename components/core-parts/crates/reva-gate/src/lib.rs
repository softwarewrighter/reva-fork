//! # reva-gate — the gateway table
//!
//! [`gate::Gate`]: one value per Rust routine of the core, each at a fixed
//! address in the gateway region (append-only: images hold the addresses).

pub mod addr;
pub mod gate;
