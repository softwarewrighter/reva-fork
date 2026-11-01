//! # reva-serve: a static site as a one-binary local web app
//!
//! The binary embeds a site (the browser REPL, plus one package such as
//! CoSy; see `build.rs`), serves it on `127.0.0.1` and opens the browser
//! on it.  Ctrl-C stops it.  See docs/serve.md.
//!
//! The port matters more than usual: the page keeps the user's files in
//! localStorage, which belongs to the origin, port included.  So the app
//! always tries its fixed port first ([`bind`]); if that is taken by a copy
//! of itself it just opens the browser there ([`Bound::Running`]), and only
//! if something else holds it does it fall back to the next free port,
//! saying that files saved there are kept apart.

#![deny(missing_docs)]

pub mod listen;
pub mod open;
pub mod site;

pub use listen::{Bound, FALLBACKS, bind, probe, serve};
pub use reva_serve_http::http;
