//! # reva-vlib-iup — the virtual library `iup`
//!
//! IUP's C functions, as Reva's `lib/ui/iup` binds them, answered over the
//! ui component's model (`Machine::ui`; docs/ui-model.md): the table of
//! functions ([`names`]), the calls ([`call`]), strings and the host's
//! data ([`data`]), and showing dialogs, the event loop and callbacks
//! ([`run`]).  No pointer crosses: elements are small integers, strings are
//! copied.  Events come from the host through `ui` host calls; a callback
//! runs as IUP ran it, a C call into the Forth word, and returns to the
//! loop through a gate.

pub mod call;
pub mod data;
pub mod names;
pub mod run;
