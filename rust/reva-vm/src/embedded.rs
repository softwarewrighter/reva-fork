//! Sources compiled into the binary, so every host (including the browser,
//! which has no file system) can boot the same Forth code.
//!
//! The boot source is `src/reva.f` itself, **verbatim**, cut just before its
//! last step, which writes a new native executable with `(save)` and exits.
//! Everything before that — the primitive words written as machine-code
//! bytes, the embedded x86 assembler, contexts, `needs`, the FFI layer and
//! the OS personality — runs unmodified on the vx86 core.  The portable
//! equivalent of that last step is a saved image (`reva --save-image`).
//!
//! Hosts see one virtual install prefix, [`INSTALL_PREFIX`]: the core
//! reports its program path as `/reva/bin/lin/reva` (the original layout:
//! `bin/<os>/reva`, `bin/help.db`, `lib/`), so `reva.f`'s `libdir` resolves
//! `needs` to `/reva/lib/...` and `helpdir` is `/reva/bin/`.  Each host maps that prefix to real
//! files (the CLI: the repository checkout or `$REVA_HOME`).

/// `src/reva.f` from this repository.
pub const REVA_F: &str = include_str!("../../../src/reva.f");

/// Where the boot source is cut: the step that saves a native executable.
pub const BOOT_MARKER: &str = "| Generate the Reva executable:";

/// Virtual install prefix seen by Forth code (see the module docs).
pub const INSTALL_PREFIX: &str = "/reva/";

/// The program path the core reports (`appname`): `appdir` is
/// `/reva/bin/lin/`, `helpdir` is `/reva/bin/` and `libdir` is `/reva/lib/`.
pub const APP_PATH: &str = "/reva/bin/lin/reva";

/// The part of `path` below [`INSTALL_PREFIX`] (`lib/util/locals` for
/// `/reva/lib/util/locals`), or `None` for any other path.
pub fn install_relative(path: &str) -> Option<&str> {
    path.strip_prefix(INSTALL_PREFIX)
}

/// Split what booting printed into reva.f's progress marks (`'1 emit` ...)
/// and the rest (normally the `Reva 2011.2 Linux` banner and the first
/// prompt; anything else means something went wrong).
pub fn split_boot_output(out: &str) -> (&str, &str) {
    let n = out
        .bytes()
        .take_while(|b| b.is_ascii_digit() || b.is_ascii_uppercase() && *b <= b'F')
        .count();
    out.split_at(n)
}

/// Virtual path under which the boot source is served.
pub const BOOT_PATH: &str = "boot/reva.f";

/// What runs in place of the cut-off tail: restart the system as the
/// saved executable would have started (see `Gate::BootRestart`), telling
/// the core where reva.f's interpreter loop is (for fault recovery).
pub const BOOT_TAIL: &str = "' ~priv.exception-reset (boot-restart)\n";

/// `reva.f` hides the compiled-string area here, just after `save` was
/// defined.  `save` hands that area to `setextra`, so the executable's
/// startup code (`getextra` ... `(compstr) !`) finds its strings again.
pub const STRINGS_MARKER: &str = "hide (compstrend)";

/// The line inserted before [`STRINGS_MARKER`]: what `save` does with the
/// string area before it writes an executable.
pub const STRINGS_SAVE: &str = "(compstr) @ (compstrend) @ over - setextra\n";

/// The boot source: `reva.f` up to [`BOOT_MARKER`] with [`STRINGS_SAVE`]
/// inserted before [`STRINGS_MARKER`], then [`BOOT_TAIL`].  These two
/// additions together stand in for "save an executable, then run it".
pub fn boot_source() -> &'static str {
    static SRC: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SRC.get_or_init(|| {
        let head = match REVA_F.find(BOOT_MARKER) {
            Some(i) => &REVA_F[..i],
            None => REVA_F,
        };
        match head.find(STRINGS_MARKER) {
            Some(j) => format!("{}{STRINGS_SAVE}{}{BOOT_TAIL}", &head[..j], &head[j..]),
            None => format!("{head}{BOOT_TAIL}"),
        }
    })
}

/// Look up an embedded file by virtual path.
pub fn file(path: &str) -> Option<&'static [u8]> {
    match path {
        BOOT_PATH => Some(boot_source().as_bytes()),
        "src/reva.f" => Some(REVA_F.as_bytes()),
        _ => None,
    }
}
