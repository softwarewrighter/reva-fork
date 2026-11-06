//! # reva-cd — Tecgraf CD (CanvasDraw) for IUP canvases
//!
//! Reva drew in IUP's `canvas[` through CD (`lib/ui/cd`): lines, boxes,
//! marks, text and bitmaps, repainted from the canvas's ACTION callback.
//! Here each CD canvas keeps a display list of [`draw::Op`]s with the
//! attributes they were drawn with; the IUP virtual library hands a dirty
//! canvas's list to the renderer as its IUP element's `DRAW` attribute,
//! in [`draw::encode`]'s text form, and the renderer paints it.
//!
//! [`draw`] (crate reva-cd-list): the operations and their text form;
//! [`state`]: canvases,
//! attributes and bitmaps; [`calls`]: the CD functions Reva programs call.

pub mod calls;
mod ops;
pub use reva_cd_list::draw;
pub mod state;
