//! The embedded site: files by path, and their media types.

/// What `build.rs` embedded: `NAME`, `PORT`, `FILES`.
pub mod embedded {
    include!(concat!(env!("OUT_DIR"), "/site.rs"));
}

/// The embedded site.
pub static SITE: Site = Site {
    name: embedded::NAME,
    files: embedded::FILES,
};

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
        format!("reva-serve {} {}\n", self.name, env!("CARGO_PKG_VERSION"))
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

#[cfg(test)]
mod tests {
    use super::*;

    static FILES: &[(&str, &[u8])] = &[
        ("a.js", b"js"),
        ("index.html", b"top"),
        ("pkg/index.html", b"pkg"),
        ("pkg/x.f", b"x"),
    ];
    const SITE: Site = Site {
        name: "t",
        files: FILES,
    };

    #[test]
    fn finds_files_and_index_pages() {
        assert_eq!(SITE.lookup("/").map(|f| f.1), Some(&b"top"[..]));
        assert_eq!(SITE.lookup("/pkg/").map(|f| f.1), Some(&b"pkg"[..]));
        assert_eq!(SITE.lookup("/pkg/x.f").map(|f| f.0), Some("pkg/x.f"));
        assert_eq!(SITE.lookup("/pkg"), None);
        assert_eq!(SITE.lookup("/nope"), None);
        assert_eq!(SITE.lookup("/pkg/../a.js"), None);
        assert_eq!(SITE.lookup("a.js"), None);
    }

    #[test]
    fn media_types() {
        assert_eq!(mime("reva.wasm"), "application/wasm");
        assert_eq!(mime("x/CoSy.f"), "text/plain; charset=utf-8");
        assert_eq!(mime("INDEX.HTML"), "text/html; charset=utf-8");
        assert_eq!(mime("noext"), "application/octet-stream");
    }
}
