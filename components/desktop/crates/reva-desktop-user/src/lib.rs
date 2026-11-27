//! # reva-desktop-user — the installed app's user folder
//!
//! An installer (scripts/make-installer.sh) ships a package read-only in
//! the install.  The user's files live in a folder of their own
//! ([`user`]): the package's `userFolder` under the platform's Documents
//! folder.  The VM sees the package's files with the user's over them,
//! and writes go to the user's folder only (docs/packaging.md).  Finding
//! the shipped package and what `--about` says of it: [`install`].

pub mod install;
pub mod user;
