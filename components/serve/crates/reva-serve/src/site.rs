//! The embedded site (reva-serve-http's `Site`, built by `build.rs`).

/// What `build.rs` embedded: `NAME`, `PORT`, `FILES`.
pub mod embedded {
    include!(concat!(env!("OUT_DIR"), "/site.rs"));
}

/// The embedded site.
pub static SITE: Site = Site {
    name: embedded::NAME,
    files: embedded::FILES,
};

pub use reva_serve_http::site::{PROBE, Site, mime};
