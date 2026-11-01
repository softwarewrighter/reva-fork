//! # reva-ui — the UI model of IUP programs
//!
//! What IUP's C functions do, as Reva's `lib/ui/iup` and `lib/ui/gui`
//! call them, over a model with no VM and no rendering
//! (docs/ui-model.md): the model and its events ([`ui`]); creating,
//! appending, showing ([`ops`]); attributes, text ones computed from the
//! edited text ([`values`]); keys, clicks and the callbacks they run
//! ([`events`]).  Renderers follow the tree's change stream.

pub mod events;
pub mod ops;
pub mod ui;
pub mod values;
