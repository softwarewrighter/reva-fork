//! # reva-ui-tree — elements and their attributes
//!
//! The element tree of the UI model ([`tree`]): building it ([`build`]),
//! reading it, attributes inherited and defaulted as IUP does ([`query`], with the rules in
//! [`tree`]), and relations ([`family`]).  Every change is recorded for renderers
//! (reva-ui-base's `Change`).

pub mod build;
pub mod family;
pub mod query;
pub mod tree;
