//! `--about`: what the app would show, read from the page built into it
//! (the release check compares it with VERSION and the commit, so an app
//! that embeds an old page fails before a release).

/// The report: the app's version, the page's build stamp (index.html's
/// `reva-build`, the About dialog), the VM's banner and whether the
/// stamp's commit is in the VM (the footer's build, from reva.wasm).
pub fn about(app_version: &str, index_html: &[u8], wasm: &[u8]) -> String {
    let stamp = between(index_html, b"<meta name=\"reva-build\" content=\"", b"\"");
    let banner =
        between(wasm, b"reva-fork 20", b" portable").map(|v| format!("reva-fork 20{v} portable"));
    let commit = stamp
        .as_deref()
        .and_then(|s| s.split(' ').nth(1))
        .map(|c| c.trim_end_matches("-dirty").to_string());
    let in_vm = commit
        .as_deref()
        .is_some_and(|c| contains(wasm, c.as_bytes()));
    let none = || "(none)".to_string();
    format!(
        "app: reva-desktop {app_version}\npage: {}\nvm: {}\nvm build: {} ({})\n",
        stamp.unwrap_or_else(none),
        banner.unwrap_or_else(none),
        commit.unwrap_or_else(none),
        if in_vm {
            "in reva.wasm"
        } else {
            "not in reva.wasm"
        },
    )
}

/// The text after `start` up to `end`, if both are there (and it is short).
fn between(hay: &[u8], start: &[u8], end: &[u8]) -> Option<String> {
    let i = find(hay, start)? + start.len();
    let n = find(&hay[i..hay.len().min(i + 200)], end)?;
    Some(String::from_utf8_lossy(&hay[i..i + n]).into_owned())
}

/// Where `needle` first occurs in `hay`.
fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Whether `hay` holds `needle` (an empty needle does not count).
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && find(hay, needle).is_some()
}
