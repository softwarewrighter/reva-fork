//! What this build calls itself: the banner Reva prints at startup, and the
//! fuller report shared by `reva -V` and the core word `version`.
//!
//! The version is the workspace version, `YYYY.MMDD.N`: the release date
//! and that day's release number, from 0 (`2026.1025.0`).  Cargo forbids
//! leading zeros, so January 5 is `2026.105.0`.  The build details come
//! from `build.rs`.  `revaver` still answers `2011.2`, the release of
//! `reva.f` this core runs, since libraries test it.

/// The fork's name.
pub const NAME: &str = "reva-fork";

/// The workspace version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The startup banner, in place of `Reva 2011.2 Linux`.
pub const BANNER: &str = concat!("reva-fork ", env!("CARGO_PKG_VERSION"), " portable");

pub const REPOSITORY: &str = "https://github.com/softwarewrighter/reva-fork";
pub const COPYRIGHT: &str = "Copyright (c) 2026 Michael A Wright";
pub const LICENSE: &str = "MIT (portable core); Reva Forth by Ron Aaron is public domain";

/// Build host, short commit and time, from `build.rs` (`unknown` if git
/// or the clock were not available).
pub const BUILD_HOST: &str = env!("REVA_BUILD_HOST");
pub const BUILD_COMMIT: &str = env!("REVA_BUILD_COMMIT");
pub const BUILD_TIMESTAMP: &str = env!("REVA_BUILD_TIMESTAMP");

/// The report's lines, without line ends.
pub fn lines() -> Vec<String> {
    vec![
        format!("{NAME} {VERSION}"),
        COPYRIGHT.into(),
        format!("License: {LICENSE}"),
        format!("Repository: {REPOSITORY}"),
        String::new(),
        "Build Information:".into(),
        format!("  Host: {BUILD_HOST}"),
        format!("  Commit: {BUILD_COMMIT}"),
        format!("  Timestamp: {BUILD_TIMESTAMP}"),
    ]
}

/// The report as text, each line ended with `\n`.
pub fn info() -> String {
    lines().iter().map(|l| format!("{l}\n")).collect()
}

/// The same facts as fields, one `key\tvalue` line each, for pages and
/// other programs (the web page's footer): `name`, `version`, `copyright`,
/// `license`, `repository`, `build_host`, `build_commit`, `build_time`.
pub fn fields() -> String {
    [
        ("name", NAME),
        ("version", VERSION),
        ("copyright", COPYRIGHT),
        ("license", LICENSE),
        ("repository", REPOSITORY),
        ("build_host", BUILD_HOST),
        ("build_commit", BUILD_COMMIT),
        ("build_time", BUILD_TIMESTAMP),
    ]
    .iter()
    .map(|(k, v)| format!("{k}\t{v}\n"))
    .collect()
}

/// Forth source for this fork's `.ver` (the banner).  Inserted into the
/// boot source right after `reva.f` defines its own `.ver`, so the startup
/// code and `-v` that follow print this one.  The report is the core word
/// `version` (`Gate::Version`), not Forth text, so the build details do not
/// change the size of the dictionary.
pub const VER_WORD: &str = concat!(
    ": .ver .\" reva-fork ",
    env!("CARGO_PKG_VERSION"),
    " portable\" ;\n"
);
