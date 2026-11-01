//! # reva-desktop-pkg — the desktop app's packages
//!
//! What does not need Tauri: a package's manifest and files
//! ([`manifest`]), branding from it ([`brand`]), VM paths under the app's
//! folder and links ([`paths`]), and the folder as files for the page
//! ([`snapshot`]).

pub mod brand;
pub mod manifest;
pub mod paths;
pub mod snapshot;
