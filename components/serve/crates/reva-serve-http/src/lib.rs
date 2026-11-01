//! # reva-serve-http — HTTP for reva-serve
//!
//! A static site ([`site::Site`]), request parsing ([`request`]), and
//! serving one connection ([`http::handle`]).

#![deny(missing_docs)]

pub mod http;
pub mod request;
pub mod site;
