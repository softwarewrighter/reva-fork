//! # reva-core-inspect — the dictionary and the machine's state
//!
//! [`words::Dictionary`]: looking words up, naming an address (for fault
//! reports and `--trace`), undoing a half-made definition;
//! [`view::View`]: what a host reads of a machine; [`config::Settings`]:
//! how it sets one up (tracing, a translator, a quiet prompt).

pub mod config;
pub mod view;
pub mod words;
