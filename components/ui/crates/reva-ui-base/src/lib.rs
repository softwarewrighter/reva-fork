//! # reva-ui-base — the UI model's vocabulary
//!
//! What the other ui crates and the renderers share: the kinds of element
//! by IUP's class names ([`kind`]), IUP's key codes and names ([`keys`]),
//! and the stream of changes a renderer applies ([`change`]), written as
//! lines of text ([`codec`]).  No dependencies (docs/ui-model.md).

pub mod change;
pub mod codec;
pub mod keys;
pub mod kind;
