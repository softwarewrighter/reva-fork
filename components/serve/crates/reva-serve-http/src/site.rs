//! A static site: files by path, and their media types.

/// The path that tells a running copy of this app apart from other servers
/// (see [`Site::probe`]).
pub const PROBE: &str = "/.reva-serve";

/// A static site: `(path, bytes)` sorted by path, paths relative, `/`-separated.
pub struct Site {
    /// The app's name (`reva`, `cosy`, ...).
    pub name: &'static str,
    /// The files, sorted by path.
    pub files: &'static [(&'static str, &'static [u8])],
}

impl Site {
    /// The file for URL path `path` (absolute, decoded): `/` and `/dir/`
    /// mean `index.html` there.  Returns the file's own path and bytes.
    pub fn lookup(&self, path: &str) -> Option<(&'static str, &'static [u8])> {
        let rel = path.strip_prefix('/')?;
        if rel.split('/').any(|s| s == "..") {
            return None;
        }
        let want = if rel.is_empty() || rel.ends_with('/') {
            format!("{rel}index.html")
        } else {
            rel.to_string()
        };
        let i = self
            .files
            .binary_search_by(|(p, _)| (*p).cmp(want.as_str()))
            .ok()?;
        Some(self.files[i])
    }

    /// What [`PROBE`] answers: `reva-serve NAME VERSION`, so a second launch
    /// can recognise the first and just open the browser on it.
    pub fn probe(&self) -> String {
        let version = include_str!("../../../../../VERSION").trim_ascii();
        format!("reva-serve {} {version}\n", self.name)
    }
}

/// The media type for a file, by extension.  `.wasm` must be
/// `application/wasm` for streaming compilation; `.f` and other Forth
/// sources are text.
pub fn mime(path: &str) -> &'static str {
    let ext = path.rsplit_once('.').map_or("", |(_, e)| e);
    match ext.to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "wasm" => "application/wasm",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "txt" | "md" | "f" | "fs" | "4th" | "csv" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
