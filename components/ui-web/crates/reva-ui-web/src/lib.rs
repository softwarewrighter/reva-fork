//! # reva-ui-web — the TUI in the web page
//!
//! A Yew element that draws reva-ui-grid's cells for the page, and the
//! desktop app, which shows the same page (docs/ui-model.md): the
//! renderer's state ([`state`]), browser keys as IUP key codes
//! ([`keys`]), the element ([`view`]), and what the page calls ([`api`]:
//! `mount`, `changes`, `resize`, `shown`, `reset`).  The page (www/tui.js)
//! passes it the change lines of the VM's `ui` `wait` calls and answers
//! them with the event lines it sends back.  Built for the page with
//! wasm-bindgen (scripts/build-web.sh), apart from the VM's own module,
//! which keeps no imports.

pub mod api;
pub mod keys;
pub mod state;
pub mod view;
