//! # reva-ui-mirror — a renderer's copy of the ui model
//!
//! A renderer in another place than the VM (the web page, a terminal, a
//! test) keeps the model's elements, attributes, shown dialogs and focus
//! by applying the change stream in order ([`mirror`], [`apply`]).  Text
//! elements' VALUE, CARET and SELECTION arrive as attributes.

pub mod apply;
pub mod mirror;
