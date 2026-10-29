//! The libraries and functions the port serves, by name.

use reva_core::{Gate, st};
use reva_vlib::{highlight, iconv, sqlite, turtle};

/// Handle `(lib)` returns for the virtual libc.  Nonzero, and not a valid
/// address, so it cannot be mistaken for a pointer.
pub const LIBC_HANDLE: u32 = 0x4c49_4243; // "LIBC"

/// Library names accepted as libc (the names `reva.f` uses per OS).
const LIBC_NAMES: &[&str] = &[
    "libc.so.6",
    "libc.so",
    "libc.dylib",
    "libc",
    "msvcrt",
    "msvcrt.dll",
];

/// C functions the virtual libc provides, by name, and their gateways.
pub const HOST_FUNCS: &[(&str, Gate)] = &[
    ("getenv", Gate::HostGetenv),
    ("putenv", Gate::HostPutenv),
    ("localtime", Gate::HostLocaltime),
    ("gmtime", Gate::HostLocaltime),
    ("opendir", Gate::HostOpendir),
    ("readdir", Gate::HostReaddir),
    ("closedir", Gate::HostClosedir),
    ("system", Gate::HostSystem),
];

/// `errno` is data, not code: `data: errno` reads through this cell.
const DATA_SYMBOLS: &[(&str, u32)] = &[("errno", st::ERRNO)];

/// The handle `(lib)` gives library `name`: a virtual library's, or 0.
pub fn lib_handle(name: &str) -> u32 {
    let is = |names: &[&str]| names.iter().any(|l| l.eq_ignore_ascii_case(name));
    if is(LIBC_NAMES) {
        LIBC_HANDLE
    } else if is(turtle::TURTLE_NAMES) {
        turtle::TURTLE_HANDLE
    } else if is(highlight::HL_NAMES) {
        highlight::HL_HANDLE
    } else if is(sqlite::SQL_NAMES) {
        sqlite::SQL_HANDLE
    } else if is(iconv::ICONV_NAMES) {
        iconv::ICONV_HANDLE
    } else {
        0
    }
}

/// The address `(func)` gives `name` in the library `handle`: a gate, a
/// data cell, or 0.
pub fn func_addr(handle: u32, name: &str) -> u32 {
    match handle {
        LIBC_HANDLE => HOST_FUNCS
            .iter()
            .find(|(f, _)| *f == name)
            .map(|(_, g)| g.addr())
            .or_else(|| {
                DATA_SYMBOLS
                    .iter()
                    .find(|(d, _)| *d == name)
                    .map(|(_, c)| *c)
            })
            .unwrap_or(0),
        turtle::TURTLE_HANDLE => turtle::lookup(name).map_or(0, Gate::addr),
        highlight::HL_HANDLE => highlight::lookup(name).map_or(0, Gate::addr),
        sqlite::SQL_HANDLE => sqlite::lookup(name).map_or(0, Gate::addr),
        iconv::ICONV_HANDLE => iconv::lookup(name).unwrap_or(0),
        _ => 0,
    }
}
