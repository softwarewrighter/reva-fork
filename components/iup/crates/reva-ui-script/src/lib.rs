//! # reva-ui-script — a headless UI host
//!
//! Answers the iup library's `ui` host calls from a script of events, for
//! tests and goldens (`reva --ui-script FILE`): the script and its lines
//! ([`script`]), the answers and the transcript of what was shown
//! ([`answer`]).  See docs/ui-model.md.

pub mod answer;
pub mod script;
