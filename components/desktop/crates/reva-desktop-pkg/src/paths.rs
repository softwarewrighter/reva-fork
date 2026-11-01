//! VM paths under the app's folder, file bytes for the page, links.

use std::path::{Component, Path, PathBuf};

/// VM paths under this prefix are the built-in library, never written out.
pub const BUILTIN: &str = "/reva/";

/// The real file for VM path `vm`, if it is a plain path under the root
/// (no `..`, not the built-in library).
pub fn real_path(root: &Path, vm: &str) -> Option<PathBuf> {
    if vm.starts_with(BUILTIN) || !vm.starts_with('/') {
        return None;
    }
    let rel = Path::new(&vm[1..]);
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(rel))
}

/// Bytes as a string of chars U+0000..U+00FF, one per byte: what the page's
/// Web Storage format already uses, and cheap to carry in JSON.
pub fn to_latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

/// The bytes of a string from [`to_latin1`] (`None`: a char above U+00FF).
pub fn from_latin1(s: &str) -> Option<Vec<u8>> {
    s.chars().map(|c| u8::try_from(c as u32).ok()).collect()
}

/// An `http://` or `https://` URL with no whitespace or control characters.
pub fn web_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    (lower.starts_with("https://") || lower.starts_with("http://"))
        && url.len() < 2048
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}
