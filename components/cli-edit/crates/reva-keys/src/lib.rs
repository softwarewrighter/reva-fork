//! # reva-keys — terminal input bytes to keys
//!
//! [`key::Key`]s from the bytes a terminal sends ([`decoder::Decoder`]):
//! UTF-8, control bytes, and the `ESC [`/`ESC O` sequences of xterm-like
//! terminals and Windows consoles with virtual terminal input.

pub mod decoder;
pub mod key;
