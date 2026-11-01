//! # reva-ui-text — editable text
//!
//! The state of IUP's text and multiline elements: the characters, the
//! caret and the selection ([`text`]); lines and columns ([`pos`]); IUP's
//! attribute forms for positions and ranges ([`attr`]); editing and caret
//! motion ([`edit`]).  Pure: no tree, no rendering.

pub mod attr;
pub mod edit;
pub mod pos;
pub mod text;
