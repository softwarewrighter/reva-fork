//! Sources compiled into the binary, so every host (including the browser,
//! which has no file system) can boot the same Forth code.
//!
//! The boot source is `src/reva.f` itself, **verbatim**, cut at the start of
//! its embedded x86 assembler (just after the `forth` that follows the
//! `| ASSEMBLER` banner, so new words default to the normal class).  Everything before that line —
//! the primitive words written as machine-code bytes, control structures,
//! classes, `does>`, `THROW:` — runs unmodified on the vx86 core.  Moving the
//! cut point later is how the port makes progress.

/// `src/reva.f` from this repository.
pub const REVA_F: &str = include_str!("../../../src/reva.f");

/// Where the boot source is cut: the first line of the assembler proper.
pub const BOOT_MARKER: &str = "| DO NOT MOVE 'REG'";

/// Virtual path under which the boot source is served.
pub const BOOT_PATH: &str = "boot/reva.f";

/// The prefix of `reva.f` that the portable core can run today.
pub fn boot_source() -> &'static str {
    match REVA_F.find(BOOT_MARKER) {
        Some(i) => &REVA_F[..i],
        None => REVA_F,
    }
}

/// Look up an embedded file by virtual path.
pub fn file(path: &str) -> Option<&'static [u8]> {
    match path {
        BOOT_PATH => Some(boot_source().as_bytes()),
        "src/reva.f" => Some(REVA_F.as_bytes()),
        _ => None,
    }
}
